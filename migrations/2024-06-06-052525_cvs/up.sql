-- Your SQL goes here
CREATE TABLE cv (
  id SERIAL PRIMARY KEY,
  application_date VARCHAR,
  job_title VARCHAR NOT NULL,
  company VARCHAR NOT NULL,
  quote VARCHAR NOT NULL,
  pdf_cv_path VARCHAR NOT NULL,
  generated BOOLEAN NOT NULL DEFAULT TRUE
)
