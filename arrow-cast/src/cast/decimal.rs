// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::cast::*;

/// A utility trait that provides checked conversions between
/// decimal types inspired by [`NumCast`]
pub(crate) trait DecimalCast: Sized {
    fn to_i128(self) -> Option<i128>;

    fn to_i256(self) -> Option<i256>;

    fn from_decimal<T: DecimalCast>(n: T) -> Option<Self>;

    fn from_f64(n: f64) -> Option<Self>;
}

impl DecimalCast for i128 {
    fn to_i128(self) -> Option<i128> {
        Some(self)
    }

    fn to_i256(self) -> Option<i256> {
        Some(i256::from_i128(self))
    }

    fn from_decimal<T: DecimalCast>(n: T) -> Option<Self> {
        n.to_i128()
    }

    fn from_f64(n: f64) -> Option<Self> {
        n.to_i128()
    }
}

impl DecimalCast for i256 {
    fn to_i128(self) -> Option<i128> {
        self.to_i128()
    }

    fn to_i256(self) -> Option<i256> {
        Some(self)
    }

    fn from_decimal<T: DecimalCast>(n: T) -> Option<Self> {
        n.to_i256()
    }

    fn from_f64(n: f64) -> Option<Self> {
        i256::from_f64(n)
    }
}

pub(crate) fn cast_decimal_to_decimal_error<I, O>(
    output_precision: u8,
    output_scale: i8,
) -> impl Fn(<I as ArrowPrimitiveType>::Native) -> ArrowError
where
    I: DecimalType,
    O: DecimalType,
    I::Native: DecimalCast + ArrowNativeTypeOp,
    O::Native: DecimalCast + ArrowNativeTypeOp,
{
    move |x: I::Native| {
        ArrowError::CastError(format!(
            "Cannot cast to {}({}, {}). Overflowing on {:?}",
            O::PREFIX,
            output_precision,
            output_scale,
            x
        ))
    }
}

pub(crate) fn convert_to_smaller_scale_decimal<I, O>(
    array: &PrimitiveArray<I>,
    input_precision: u8,
    input_scale: i8,
    output_precision: u8,
    output_scale: i8,
    cast_options: &CastOptions,
) -> Result<PrimitiveArray<O>, ArrowError>
where
    I: DecimalType,
    O: DecimalType,
    I::Native: DecimalCast + ArrowNativeTypeOp,
    O::Native: DecimalCast + ArrowNativeTypeOp,
{
    let error = cast_decimal_to_decimal_error::<I, O>(output_precision, output_scale);
    let delta_scale = input_scale - output_scale;
    // if the reduction of the input number through scaling (dividing) is greater
    // than a possible precision loss (plus potential increase via rounding)
    // every input number will fit into the output type
    // Example: If we are starting with any number of precision 5 [xxxxx],
    // then and decrease the scale by 3 will have the following effect on the representation:
    // [xxxxx] -> [xx] (+ 1 possibly, due to rounding).
    // The rounding may add an additional digit, so the cast to be infallible,
    // the output type needs to have at least 3 digits of precision.
    // e.g. Decimal(5, 3) 99.999 to Decimal(3, 0) will result in 100:
    // [99999] -> [99] + 1 = [100], a cast to Decimal(2, 0) would not be possible
    let is_infallible_cast = (input_precision as i8) - delta_scale < (output_precision as i8);

    let div = I::Native::from_decimal(10_i128)
        .unwrap()
        .pow_checked(delta_scale as u32)?;

    let half = div.div_wrapping(I::Native::from_usize(2).unwrap());
    let half_neg = half.neg_wrapping();

    let f = |x: I::Native| {
        // div is >= 10 and so this cannot overflow
        let d = x.div_wrapping(div);
        let r = x.mod_wrapping(div);

        // Round result
        let adjusted = match x >= I::Native::ZERO {
            true if r >= half => d.add_wrapping(I::Native::ONE),
            false if r <= half_neg => d.sub_wrapping(I::Native::ONE),
            _ => d,
        };
        O::Native::from_decimal(adjusted)
    };

    Ok(if is_infallible_cast {
        // make sure we don't perform calculations that don't make sense w/o validation
        validate_decimal_precision_and_scale::<O>(output_precision, output_scale)?;
        let g = |x: I::Native| f(x).unwrap(); // unwrapping is safe since the result is guaranteed
                                              // to fit into the target type
        array.unary(g)
    } else if cast_options.safe {
        array.unary_opt(|x| f(x).filter(|v| O::is_valid_decimal_precision(*v, output_precision)))
    } else {
        array.try_unary(|x| {
            f(x).ok_or_else(|| error(x))
                .and_then(|v| O::validate_decimal_precision(v, output_precision).map(|_| v))
        })?
    })
}

pub(crate) fn convert_to_bigger_or_equal_scale_decimal<I, O>(
    array: &PrimitiveArray<I>,
    input_precision: u8,
    input_scale: i8,
    output_precision: u8,
    output_scale: i8,
    cast_options: &CastOptions,
) -> Result<PrimitiveArray<O>, ArrowError>
where
    I: DecimalType,
    O: DecimalType,
    I::Native: DecimalCast + ArrowNativeTypeOp,
    O::Native: DecimalCast + ArrowNativeTypeOp,
{
    let error = cast_decimal_to_decimal_error::<I, O>(output_precision, output_scale);
    let delta_scale = output_scale - input_scale;
    let mul = O::Native::from_decimal(10_i128)
        .unwrap()
        .pow_checked(delta_scale as u32)?;

    // if the gain in precision (digits) is greater than the multiplication due to scaling
    // every number will fit into the output type
    // Example: If we are starting with any number of precision 5 [xxxxx],
    // then an increase of scale by 3 will have the following effect on the representation:
    // [xxxxx] -> [xxxxx000], so for the cast to be infallible, the output type
    // needs to provide at least 8 digits precision
    let is_infallible_cast = (input_precision as i8) + delta_scale <= (output_precision as i8);
    let f = |x| O::Native::from_decimal(x).and_then(|x| x.mul_checked(mul).ok());

    Ok(if is_infallible_cast {
        // make sure we don't perform calculations that don't make sense w/o validation
        validate_decimal_precision_and_scale::<O>(output_precision, output_scale)?;
        // unwrapping is safe since the result is guaranteed to fit into the target type
        let f = |x| O::Native::from_decimal(x).unwrap().mul_wrapping(mul);
        array.unary(f)
    } else if cast_options.safe {
        array.unary_opt(|x| f(x).filter(|v| O::is_valid_decimal_precision(*v, output_precision)))
    } else {
        array.try_unary(|x| {
            f(x).ok_or_else(|| error(x))
                .and_then(|v| O::validate_decimal_precision(v, output_precision).map(|_| v))
        })?
    })
}

// Only support one type of decimal cast operations
pub(crate) fn cast_decimal_to_decimal_same_type<T>(
    array: &PrimitiveArray<T>,
    input_precision: u8,
    input_scale: i8,
    output_precision: u8,
    output_scale: i8,
    cast_options: &CastOptions,
) -> Result<ArrayRef, ArrowError>
where
    T: DecimalType,
    T::Native: DecimalCast + ArrowNativeTypeOp,
{
    let array: PrimitiveArray<T> =
        if input_scale == output_scale && input_precision <= output_precision {
            array.clone()
        } else if input_scale <= output_scale {
            convert_to_bigger_or_equal_scale_decimal::<T, T>(
                array,
                input_precision,
                input_scale,
                output_precision,
                output_scale,
                cast_options,
            )?
        } else {
            // input_scale > output_scale
            convert_to_smaller_scale_decimal::<T, T>(
                array,
                input_precision,
                input_scale,
                output_precision,
                output_scale,
                cast_options,
            )?
        };

    Ok(Arc::new(array.with_precision_and_scale(
        output_precision,
        output_scale,
    )?))
}

// Support two different types of decimal cast operations
pub(crate) fn cast_decimal_to_decimal<I, O>(
    array: &PrimitiveArray<I>,
    input_precision: u8,
    input_scale: i8,
    output_precision: u8,
    output_scale: i8,
    cast_options: &CastOptions,
) -> Result<ArrayRef, ArrowError>
where
    I: DecimalType,
    O: DecimalType,
    I::Native: DecimalCast + ArrowNativeTypeOp,
    O::Native: DecimalCast + ArrowNativeTypeOp,
{
    let array: PrimitiveArray<O> = if input_scale > output_scale {
        convert_to_smaller_scale_decimal::<I, O>(
            array,
            input_precision,
            input_scale,
            output_precision,
            output_scale,
            cast_options,
        )?
    } else {
        convert_to_bigger_or_equal_scale_decimal::<I, O>(
            array,
            input_precision,
            input_scale,
            output_precision,
            output_scale,
            cast_options,
        )?
    };

    Ok(Arc::new(array.with_precision_and_scale(
        output_precision,
        output_scale,
    )?))
}

/// Parses given string to specified decimal native (i128/i256) based on given
/// scale. Returns an `Err` if it cannot parse given string.
pub(crate) fn parse_string_to_decimal_native<T: DecimalType>(
    value_str: &str,
    scale: usize,
) -> Result<T::Native, ArrowError>
where
    T::Native: DecimalCast + ArrowNativeTypeOp,
{
    let value_str = value_str.trim();
    let parts: Vec<&str> = value_str.split('.').collect();
    if parts.len() > 2 {
        return Err(ArrowError::InvalidArgumentError(format!(
            "Invalid decimal format: {value_str:?}"
        )));
    }

    let (negative, first_part) = if parts[0].is_empty() {
        (false, parts[0])
    } else {
        match parts[0].as_bytes()[0] {
            b'-' => (true, &parts[0][1..]),
            b'+' => (false, &parts[0][1..]),
            _ => (false, parts[0]),
        }
    };

    let integers = first_part;
    let decimals = if parts.len() == 2 { parts[1] } else { "" };

    if !integers.is_empty() && !integers.as_bytes()[0].is_ascii_digit() {
        return Err(ArrowError::InvalidArgumentError(format!(
            "Invalid decimal format: {value_str:?}"
        )));
    }

    if !decimals.is_empty() && !decimals.as_bytes()[0].is_ascii_digit() {
        return Err(ArrowError::InvalidArgumentError(format!(
            "Invalid decimal format: {value_str:?}"
        )));
    }

    // Adjust decimal based on scale
    let mut number_decimals = if decimals.len() > scale {
        let decimal_number = i256::from_string(decimals).ok_or_else(|| {
            ArrowError::InvalidArgumentError(format!("Cannot parse decimal format: {value_str}"))
        })?;

        let div = i256::from_i128(10_i128).pow_checked((decimals.len() - scale) as u32)?;

        let half = div.div_wrapping(i256::from_i128(2));
        let half_neg = half.neg_wrapping();

        let d = decimal_number.div_wrapping(div);
        let r = decimal_number.mod_wrapping(div);

        // Round result
        let adjusted = match decimal_number >= i256::ZERO {
            true if r >= half => d.add_wrapping(i256::ONE),
            false if r <= half_neg => d.sub_wrapping(i256::ONE),
            _ => d,
        };

        let integers = if !integers.is_empty() {
            i256::from_string(integers)
                .ok_or_else(|| {
                    ArrowError::InvalidArgumentError(format!(
                        "Cannot parse decimal format: {value_str}"
                    ))
                })
                .map(|v| v.mul_wrapping(i256::from_i128(10_i128).pow_wrapping(scale as u32)))?
        } else {
            i256::ZERO
        };

        format!("{}", integers.add_wrapping(adjusted))
    } else {
        let padding = if scale > decimals.len() { scale } else { 0 };

        let decimals = format!("{decimals:0<padding$}");
        format!("{integers}{decimals}")
    };

    if negative {
        number_decimals.insert(0, '-');
    }

    let value = i256::from_string(number_decimals.as_str()).ok_or_else(|| {
        ArrowError::InvalidArgumentError(format!(
            "Cannot convert {} to {}: Overflow",
            value_str,
            T::PREFIX
        ))
    })?;

    T::Native::from_decimal(value).ok_or_else(|| {
        ArrowError::InvalidArgumentError(format!("Cannot convert {} to {}", value_str, T::PREFIX))
    })
}

pub(crate) fn generic_string_to_decimal_cast<'a, T, S>(
    from: &'a S,
    precision: u8,
    scale: i8,
    cast_options: &CastOptions,
) -> Result<PrimitiveArray<T>, ArrowError>
where
    T: DecimalType,
    T::Native: DecimalCast + ArrowNativeTypeOp,
    &'a S: StringArrayType<'a>,
{
    if cast_options.safe {
        let iter = from.iter().map(|v| {
            v.and_then(|v| parse_string_to_decimal_native::<T>(v, scale as usize).ok())
                .and_then(|v| T::is_valid_decimal_precision(v, precision).then_some(v))
        });
        // Benefit:
        //     20% performance improvement
        // Soundness:
        //     The iterator is trustedLen because it comes from an `StringArray`.
        Ok(unsafe {
            PrimitiveArray::<T>::from_trusted_len_iter(iter)
                .with_precision_and_scale(precision, scale)?
        })
    } else {
        let vec = from
            .iter()
            .map(|v| {
                v.map(|v| {
                    parse_string_to_decimal_native::<T>(v, scale as usize)
                        .map_err(|_| {
                            ArrowError::CastError(format!(
                                "Cannot cast string '{}' to value of {:?} type",
                                v,
                                T::DATA_TYPE,
                            ))
                        })
                        .and_then(|v| T::validate_decimal_precision(v, precision).map(|_| v))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        // Benefit:
        //     20% performance improvement
        // Soundness:
        //     The iterator is trustedLen because it comes from an `StringArray`.
        Ok(unsafe {
            PrimitiveArray::<T>::from_trusted_len_iter(vec.iter())
                .with_precision_and_scale(precision, scale)?
        })
    }
}

pub(crate) fn string_to_decimal_cast<T, Offset: OffsetSizeTrait>(
    from: &GenericStringArray<Offset>,
    precision: u8,
    scale: i8,
    cast_options: &CastOptions,
) -> Result<PrimitiveArray<T>, ArrowError>
where
    T: DecimalType,
    T::Native: DecimalCast + ArrowNativeTypeOp,
{
    generic_string_to_decimal_cast::<T, GenericStringArray<Offset>>(
        from,
        precision,
        scale,
        cast_options,
    )
}

pub(crate) fn string_view_to_decimal_cast<T>(
    from: &StringViewArray,
    precision: u8,
    scale: i8,
    cast_options: &CastOptions,
) -> Result<PrimitiveArray<T>, ArrowError>
where
    T: DecimalType,
    T::Native: DecimalCast + ArrowNativeTypeOp,
{
    generic_string_to_decimal_cast::<T, StringViewArray>(from, precision, scale, cast_options)
}

/// Cast Utf8 to decimal
pub(crate) fn cast_string_to_decimal<T, Offset: OffsetSizeTrait>(
    from: &dyn Array,
    precision: u8,
    scale: i8,
    cast_options: &CastOptions,
) -> Result<ArrayRef, ArrowError>
where
    T: DecimalType,
    T::Native: DecimalCast + ArrowNativeTypeOp,
{
    if scale < 0 {
        return Err(ArrowError::InvalidArgumentError(format!(
            "Cannot cast string to decimal with negative scale {scale}"
        )));
    }

    if scale > T::MAX_SCALE {
        return Err(ArrowError::InvalidArgumentError(format!(
            "Cannot cast string to decimal greater than maximum scale {}",
            T::MAX_SCALE
        )));
    }

    let result = match from.data_type() {
        DataType::Utf8View => string_view_to_decimal_cast::<T>(
            from.as_any().downcast_ref::<StringViewArray>().unwrap(),
            precision,
            scale,
            cast_options,
        )?,
        DataType::Utf8 | DataType::LargeUtf8 => string_to_decimal_cast::<T, Offset>(
            from.as_any()
                .downcast_ref::<GenericStringArray<Offset>>()
                .unwrap(),
            precision,
            scale,
            cast_options,
        )?,
        other => {
            return Err(ArrowError::ComputeError(format!(
                "Cannot cast {other:?} to decimal",
            )))
        }
    };

    Ok(Arc::new(result))
}

pub(crate) fn cast_floating_point_to_decimal<T: ArrowPrimitiveType, D>(
    array: &PrimitiveArray<T>,
    precision: u8,
    scale: i8,
    cast_options: &CastOptions,
) -> Result<ArrayRef, ArrowError>
where
    <T as ArrowPrimitiveType>::Native: AsPrimitive<f64>,
    D: DecimalType + ArrowPrimitiveType,
    <D as ArrowPrimitiveType>::Native: DecimalCast,
{
    let mul = 10_f64.powi(scale as i32);

    if cast_options.safe {
        array
            .unary_opt::<_, D>(|v| {
                D::Native::from_f64((mul * v.as_()).round())
                    .filter(|v| D::is_valid_decimal_precision(*v, precision))
            })
            .with_precision_and_scale(precision, scale)
            .map(|a| Arc::new(a) as ArrayRef)
    } else {
        array
            .try_unary::<_, D, _>(|v| {
                D::Native::from_f64((mul * v.as_()).round())
                    .ok_or_else(|| {
                        ArrowError::CastError(format!(
                            "Cannot cast to {}({}, {}). Overflowing on {:?}",
                            D::PREFIX,
                            precision,
                            scale,
                            v
                        ))
                    })
                    .and_then(|v| D::validate_decimal_precision(v, precision).map(|_| v))
            })?
            .with_precision_and_scale(precision, scale)
            .map(|a| Arc::new(a) as ArrayRef)
    }
}

pub(crate) fn cast_decimal_to_integer<D, T>(
    array: &dyn Array,
    base: D::Native,
    scale: i8,
    cast_options: &CastOptions,
) -> Result<ArrayRef, ArrowError>
where
    T: ArrowPrimitiveType,
    <T as ArrowPrimitiveType>::Native: NumCast,
    D: DecimalType + ArrowPrimitiveType,
    <D as ArrowPrimitiveType>::Native: ArrowNativeTypeOp + ToPrimitive,
{
    let array = array.as_primitive::<D>();

    let div: D::Native = base.pow_checked(scale as u32).map_err(|_| {
        ArrowError::CastError(format!(
            "Cannot cast to {:?}. The scale {} causes overflow.",
            D::PREFIX,
            scale,
        ))
    })?;

    let mut value_builder = PrimitiveBuilder::<T>::with_capacity(array.len());

    if cast_options.safe {
        for i in 0..array.len() {
            if array.is_null(i) {
                value_builder.append_null();
            } else {
                let v = array
                    .value(i)
                    .div_checked(div)
                    .ok()
                    .and_then(<T::Native as NumCast>::from::<D::Native>);

                value_builder.append_option(v);
            }
        }
    } else {
        for i in 0..array.len() {
            if array.is_null(i) {
                value_builder.append_null();
            } else {
                let v = array.value(i).div_checked(div)?;

                let value = <T::Native as NumCast>::from::<D::Native>(v).ok_or_else(|| {
                    ArrowError::CastError(format!(
                        "value of {:?} is out of range {}",
                        v,
                        T::DATA_TYPE
                    ))
                })?;

                value_builder.append_value(value);
            }
        }
    }
    Ok(Arc::new(value_builder.finish()))
}

// Cast the decimal array to floating-point array
pub(crate) fn cast_decimal_to_float<D: DecimalType, T: ArrowPrimitiveType, F>(
    array: &dyn Array,
    op: F,
) -> Result<ArrayRef, ArrowError>
where
    F: Fn(D::Native) -> T::Native,
{
    let array = array.as_primitive::<D>();
    let array = array.unary::<_, T>(op);
    Ok(Arc::new(array))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_to_decimal_native() -> Result<(), ArrowError> {
        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("0", 0)?,
            0_i128
        );
        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("0", 5)?,
            0_i128
        );

        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("123", 0)?,
            123_i128
        );
        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("123", 5)?,
            12300000_i128
        );

        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("123.45", 0)?,
            123_i128
        );
        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("123.45", 5)?,
            12345000_i128
        );

        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("123.4567891", 0)?,
            123_i128
        );
        assert_eq!(
            parse_string_to_decimal_native::<Decimal128Type>("123.4567891", 5)?,
            12345679_i128
        );
        Ok(())
    }
}
