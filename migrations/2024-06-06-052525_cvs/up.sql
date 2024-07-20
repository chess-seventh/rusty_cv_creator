-- Your SQL goes here
CREATE TABLE cv (
  id INTEGER NOT NULL PRIMARY KEY,
  application_date VARCHAR,
  job_title VARCHAR NOT NULL,
  company VARCHAR NOT NULL,
  quote VARCHAR NOT NULL,
  pdf_cv_path VARCHAR NOT NULL,
  pdf_cv BLOB NOT NULL,
  generated BOOLEAN NOT NULL DEFAULT TRUE
)
