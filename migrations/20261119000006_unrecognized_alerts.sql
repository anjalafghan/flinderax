-- Bank texts the parser could not read are kept for 30 days instead of vanishing, so the missing
-- transaction can be added by hand and the text can become a parser test case.
CREATE TABLE unrecognized_alerts (
    alert_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'sms',
    received_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, text)
);
CREATE INDEX unrecognized_alerts_user ON unrecognized_alerts (user_id, received_at DESC);
