# Shared semantic vocabulary for generated machine-artifact contracts.
# Rust owns validation and generation. Nickel is the typed review authority.
let Predicate = fun predicate => std.contract.from_predicate predicate in
let DigestHexCharacters = 64 in
let DigestAlgorithmPrefixCharacters = 7 in
let Base64DecodedBytesPerQuantum = 3 in
let Base64EncodedCharactersPerQuantum = 4 in
let SafeReferenceBytesMax = 4096 in
let RedactionTextBytesMax = 16384 in
let LowerHex64 = fun value =>
  std.is_string value
  && std.string.length value == DigestHexCharacters
  && std.string.is_match "^[0-9a-f]+$" value
in
let PrefixedLowerHex64 = fun value =>
  std.is_string value
  && std.string.length value == DigestAlgorithmPrefixCharacters + DigestHexCharacters
  && std.string.is_match "^(blake3|sha256):[0-9a-f]+$" value
in
let utf8_byte_length = fun value =>
  let encoded_length = std.string.length (std.string.base64_encode 'NoPad value) in
  std.number.floor
    (encoded_length * Base64DecodedBytesPerQuantum / Base64EncodedCharactersPerQuantum)
in
let SafeReferencePredicate = fun value =>
  std.is_string value
  && utf8_byte_length value > 0
  && utf8_byte_length value <= SafeReferenceBytesMax
  && std.string.is_match m%"^[^\x00-\x1f\x7f\\]+$"% value
  && !(std.array.elem ".." (std.string.split "/" value))
in
let RedactionSafePredicate = fun value =>
  if !std.is_string value then false else
  let lower = std.string.lowercase value in
  utf8_byte_length value <= RedactionTextBytesMax
  && std.string.is_match m%"^[^\x00]*$"% value
  && std.array.all
    (fun fragment => !(std.string.contains fragment lower))
    ["bearer ", "access_token", "access-token", "secret=", "password=", "private_key", "private-key"]
in
{
  FromPredicate = Predicate,
  Utf8ByteLength = utf8_byte_length,
  IsNonEmptyString = fun value => std.is_string value && std.string.length value > 0,
  IsBlake3Hex = LowerHex64,
  IsSha256Hex = LowerHex64,
  IsMantleDigest = fun value => LowerHex64 value || PrefixedLowerHex64 value,
  IsSafeReference = SafeReferencePredicate,
  IsRedactionSafe = RedactionSafePredicate,
  Null = Predicate (fun value => value == null),
  Integer = Predicate (fun value => std.is_number value && std.number.is_integer value),
  Exact = fun expected => Predicate (fun value => value == expected),
  StringEnum = fun allowed => Predicate (fun value => std.is_string value && std.array.elem value allowed),
  NumberBounds = fun minimum maximum => Predicate (fun value => value >= minimum && value <= maximum),
  StringBounds = fun minimum maximum => Predicate (fun value =>
    let length_bytes = utf8_byte_length value in length_bytes >= minimum && length_bytes <= maximum
  ),
  ArrayBounds = fun minimum maximum => Predicate (fun value =>
    let length = std.array.length value in length >= minimum && length <= maximum
  ),
  RecordBounds = fun minimum maximum => Predicate (fun value =>
    let length = std.array.length (std.record.fields value) in length >= minimum && length <= maximum
  ),
  NonEmptyString = Predicate (fun value => std.is_string value && std.string.length value > 0),
  Blake3Hex = Predicate LowerHex64,
  Sha256Hex = Predicate LowerHex64,
  MantleDigest = Predicate (fun value => LowerHex64 value || PrefixedLowerHex64 value),
  SafeReference = Predicate SafeReferencePredicate,
  RedactionSafe = Predicate RedactionSafePredicate,
  CrossField = fun predicate => Predicate predicate,
}
