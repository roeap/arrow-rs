searchState.loadedDescShard("arrow_cast", 0, "Functions for converting from one data type to another in …\nFunctions for converting data in <code>GenericBinaryArray</code> such …\nCast kernels to convert <code>ArrayRef</code>  between supported …\nFunctions for printing array values as human-readable …\n<code>Parser</code> implementations for converting strings to Arrow …\nUtilities for pretty printing <code>RecordBatch</code>es and <code>Array</code>s.\nA GeneralPurpose engine using the alphabet::STANDARD …\nA GeneralPurpose engine using the alphabet::STANDARD …\nA GeneralPurpose engine using the alphabet::URL_SAFE …\nA GeneralPurpose engine using the alphabet::URL_SAFE …\nThe config type used by this engine\nThe decode estimate used by this engine\nAn <code>Engine</code> provides low-level encoding and decoding …\nBase64 decode each element of <code>array</code> with the provided …\nBas64 encode each element of <code>array</code> with the provided <code>Engine</code>\nReturns the config for this engine.\nDecode the input into a new <code>Vec</code>.\nDecode the input into the provided output slice.\nDecode the input into the provided output slice.\nDecode the <code>input</code> into the supplied <code>buffer</code>.\nEncode arbitrary octets as base64 using the provided <code>Engine</code>…\nEncode arbitrary octets as base64 into a supplied slice. …\nEncode arbitrary octets as base64 into a supplied <code>String</code>. …\nCastOptions provides a way to override the default cast …\nReturn true if a value of type <code>from_type</code> can be cast into …\nCast <code>array</code> to the provided data type and return a new …\nHelper function to cast from one <code>BinaryArray</code> or ‘…\nCast Boolean types to numeric\nHelper function to cast from one <code>ByteArrayType</code> to another …\nCast the array from duration and interval\nHelper function to cast from ‘FixedSizeBinaryArray’ to …\nCast the array from interval day time to month day nano\nCast the array from interval year month to month day nano\nCast the array from interval to duration\nConvert Array into a PrimitiveArray of type, and apply …\nCast numeric types to Boolean\nCast the primitive array using …\nHelper function to cast from one <code>ByteViewType</code> array to …\nTry to cast <code>array</code> to <code>to_type</code> if possible.\nFormatting options when casting from temporal types to …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nhow to handle cast failures, either return NULL …\nGet the time unit as a multiple of a second\nA utility trait that provides checked conversions between …\nCast Utf8 to decimal\nParses given string to specified decimal native …\nAttempts to encode an array into an <code>ArrayDictionary</code> with …\nAttempts to cast an <code>ArrayDictionary</code> with index type K into …\nPack a data type into a dictionary array passing the …\nCast the container type of List/Largelist array along with …\nHelper function that takes an Generic list container and …\nHelper function that takes a primitive array and casts to …\nHelper function that takes a primitive array and casts to …\nHelper function that takes a map container and casts the …\nGets the key field from the entries of a map.  For all …\nGets the value field from the entries of a map.  For all …\nA specified helper to cast from <code>GenericBinaryArray</code> to …\nCasts generic string arrays to an ArrowTimestampType …\nCasts Utf8 to Boolean\nCasts string view arrays to an ArrowTimestampType …\nParse UTF-8\nParse UTF-8 View\nA string formatter for an <code>Array</code>\n<code>Display</code> but accepting an index\n<code>DisplayIndex</code> with additional state\nFormat for displaying durations\nContains the error value\nPairs a boxed <code>DisplayIndex</code> with its field name\nEither an <code>ArrowError</code> or <code>std::fmt::Error</code>\nOptions for formatting arrays\nISO 8601 - <code>P198DT72932.972880S</code>\nNo value.\nContains the success value\nA human readable representation - …\nSome value of type <code>T</code>.\nImplements <code>Display</code> for a specific array value\nGet the value at the given row in an array as a String.\nDate format for date arrays\nFormat for DateTime arrays\nDuration format\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConverts numeric type to a <code>String</code>\nFormat string for nulls\nIf set to <code>true</code> any formatting errors will be written to …\nTime format for time arrays\nTimestamp format for timestamp arrays\nTimestamp format for timestamp with timezone arrays\nReturns an <code>ArrayFormatter</code> that can be used to format <code>array</code>\nFallibly converts this to a string\nReturns a <code>ValueFormatter</code> that implements <code>Display</code> for the …\nOverrides the format used for <code>DataType::Date32</code> columns\nOverrides the format used for <code>DataType::Date64</code> columns\nIf set to <code>true</code> any formatting errors will be written to …\nOverrides the format used for duration columns\nOverrides the string used to represent a null\nOverrides the format used for <code>DataType::Time32</code> and …\nOverrides the format used for <code>DataType::Timestamp</code> columns …\nOverrides the format used for <code>DataType::Timestamp</code> columns …\nWrites this value to the provided <code>Write</code>\nNumber of days between 0001-01-01 and 1970-01-01\nError message if nanosecond conversion request beyond …\nChosen based on the number of decimal digits in 1 week in …\nSpecialized parsing implementations to convert strings to …\nHelper for parsing RFC3339 timestamps\nInterval addition following Postgres behavior. Fractional …\nParses a date of the form <code>1997-01-31</code>\nThe default unit to use if none is specified e.g. …\nThe timestamp bytes to parse minus <code>b&#39;0&#39;</code>\nThe fractional component multiplied by …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nThe integer component of the interval amount\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThis API is only stable since 1.70 so can’t use it when …\nA mask containing a <code>1</code> bit where the corresponding byte is …\ntest if a character is NOT part of an interval numeric …\nParse string value in traditional Postgres format such as …\nParse the string format decimal value to i128/i256 format …\nparse the string into a vector of interval components i.e. …\nParse nanoseconds from the first <code>N</code> values in digits, …\nSplit an interval into a vec of amounts and units.\nAccepts a string and parses it relative to the provided …\nAccepts a string in ISO8601 standard format and some …\nAccepts a string in RFC3339 / ISO8601 standard format and …\nReturns true if the byte at <code>idx</code> in the original string …\nParses a time of any of forms\nFallible conversion of <code>NaiveDateTime</code> to <code>i64</code> nanoseconds\nConvert a series of record batches into a table\nCreate a visual representation of record batches\nCreate a visual representation of record batches\nCreate a visual representation of columns\nPrints a visual representation of record batches to stdout\nPrints a visual representation of a list of column to …")