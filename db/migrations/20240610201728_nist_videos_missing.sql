-- These two additional columns are not in the original NIST database.
-- They are added to track videos/tapes that are in the database but were not included in the
-- released material. The additional notes column is to track the reason why the tape is missing, or
-- any other information. It's distinct from the `notes` column to preserve those as NIST's original
-- notes.
ALTER TABLE nist_videos
ADD COLUMN is_missing BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN additional_notes TEXT;
