-- Add migration script here

ALTER TABLE semester
    MODIFY COLUMN start_date date NOT NULL,
    MODIFY COLUMN end_date date NOT NULL;

ALTER TABLE event
    MODIFY COLUMN call_time timestamp NOT NULL,
    MODIFY COLUMN release_time timestamp NULL DEFAULT NULL;

ALTER TABLE gig
    MODIFY COLUMN performance_time timestamp NOT NULL;

ALTER TABLE gig_request
    MODIFY COLUMN start_time timestamp NOT NULL;
