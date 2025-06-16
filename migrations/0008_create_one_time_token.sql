CREATE TABLE one_time_tokens (
  token         TEXT PRIMARY KEY,
  used          BOOLEAN NOT NULL DEFAULT FALSE,
  created_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
  status_code   INTEGER,
  response_body BLOB,          -- e.g. JSON bytes
  response_headers TEXT        -- e.g. JSON-serialized map of headers
);
