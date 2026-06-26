CREATE TABLE IF NOT EXISTS public_lead_intake_submissions (
    id             BIGSERIAL PRIMARY KEY,
    email          TEXT NOT NULL,
    event_type     TEXT NOT NULL,
    lead_source    TEXT NOT NULL,
    payload        JSONB NOT NULL,
    forward_status TEXT NOT NULL,
    forward_detail TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_public_lead_intake_submissions_created_at
    ON public_lead_intake_submissions (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_public_lead_intake_submissions_email
    ON public_lead_intake_submissions (email);

CREATE INDEX IF NOT EXISTS idx_public_lead_intake_submissions_event_type
    ON public_lead_intake_submissions (event_type);
