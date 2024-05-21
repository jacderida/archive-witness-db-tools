CREATE TABLE nist_tapes_release_files (
    nist_tape_id INTEGER NOT NULL REFERENCES nist_tapes(tape_id),
    release_file_id INTEGER NOT NULL REFERENCES release_files(id),
    PRIMARY KEY (nist_tape_id, release_file_id)
);
