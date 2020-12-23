use base64::decode;
use std::str;
use thiserror::Error;
use ureq::SerdeValue;

#[derive(PartialEq, Debug, Error)]
pub enum DecoderError {
    #[error("JSON has no content field")]
    NoContent,

    #[error("JSON content field is not a string {0}")]
    InvalidContentType(SerdeValue),

    #[error("JSON content field is present but is not valid base64 {0}")]
    InvalidContent(base64::DecodeError),

    #[error("Invalid content encoding, result string is not valid utf-8 {0}")]
    InvalidContentEncoding(str::Utf8Error),
}

pub fn decode_content(response: SerdeValue) -> Result<String, DecoderError> {
    match &response["content"] {
        ureq::SerdeValue::String(base_64_content) => match decode(base_64_content) {
            Ok(decoded_content) => match str::from_utf8(&decoded_content) {
                Ok(decoded_str) => Ok(decoded_str.to_string()),
                Err(err) => Err(DecoderError::InvalidContentEncoding(err)),
            },
            Err(err) => Err(DecoderError::InvalidContent(err)),
        },
        ureq::SerdeValue::Null => Err(DecoderError::NoContent),
        _ => Err(DecoderError::InvalidContentType(
            response["content"].clone(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::encode;
    use ureq::{SerdeMap, SerdeValue};

    #[test]
    fn can_decode_json_with_base64_content_correctly() -> Result<(), Box<dyn std::error::Error>> {
        let content = encode(b"my content");
        let mut valid_response = SerdeMap::new();
        valid_response.insert("content".to_string(), SerdeValue::String(content));
        let json_response = SerdeValue::Object(valid_response);

        let decoded_content = decode_content(json_response)?;

        assert_eq!("my content", decoded_content);
        Ok(())
    }

    #[test]
    fn cannot_decode_json_that_doesnt_have_content() {
        let json_response = SerdeValue::Object(SerdeMap::new());

        let content_error = decode_content(json_response);

        assert!(content_error.is_err());
        if let Err(result_error) = content_error {
            assert_eq!(result_error, DecoderError::NoContent)
        }
    }

    #[test]
    fn cannot_decode_json_that_has_invalid_content() {
        let mut invalid_response = SerdeMap::new();
        invalid_response.insert("content".to_string(), SerdeValue::Bool(false));
        let json_response = SerdeValue::Object(invalid_response);

        let content_error = decode_content(json_response);

        assert!(content_error.is_err());
        if let Err(result_error) = content_error {
            assert_eq!(
                result_error,
                DecoderError::InvalidContentType(SerdeValue::Bool(false))
            )
        }
    }

    #[test]
    fn cannot_base64_decode_json_content() {
        let decoded_content = "my content".to_string();
        let mut valid_response = SerdeMap::new();
        valid_response.insert("content".to_string(), SerdeValue::String(decoded_content));
        let json_response = SerdeValue::Object(valid_response);

        let decoding_error = decode_content(json_response);

        assert!(decoding_error.is_err());
        if let Err(actual_error) = decoding_error {
            assert_eq!(
                actual_error,
                DecoderError::InvalidContent(base64::DecodeError::InvalidByte(2, 32))
            )
        }
    }

    #[test]
    fn when_given_invalid_ut8_in_content() {
        let invalid_byte_stream = vec![237, 166, 164];
        let content = encode(&invalid_byte_stream);
        let mut valid_response = SerdeMap::new();
        valid_response.insert("content".to_string(), SerdeValue::String(content));
        let json_response = SerdeValue::Object(valid_response);

        let invalid_utf8_err = decode_content(json_response);

        let expected_utf_error = String::from_utf8(invalid_byte_stream)
            .unwrap_err()
            .utf8_error();

        assert!(invalid_utf8_err.is_err());
        if let Err(actual_error) = invalid_utf8_err {
            assert_eq!(
                actual_error,
                DecoderError::InvalidContentEncoding(expected_utf_error)
            )
        }
    }
}
