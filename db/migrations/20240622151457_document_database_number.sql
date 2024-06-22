-- This column wasn't in NIST's original database, but it was provided as part of a table in a FOIA
-- response.
ALTER TABLE nist_tapes
ADD COLUMN document_database_number VARCHAR(20);
