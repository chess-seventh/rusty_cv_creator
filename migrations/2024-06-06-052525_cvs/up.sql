-- Your SQL goes here
CREATE TABLE cvs (
  id INT PRIMARY KEY NOT NULL,
  job_title VARCHAR NOT NULL,
  company VARCHAR NOT NULL,
  quote VARCHAR,
  generated BOOLEAN NOT NULL DEFAULT TRUE
)
