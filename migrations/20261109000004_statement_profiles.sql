-- A statement layout the user mapped once, found again by its header signature (see
-- statement::header_signature). The mapping is JSON; the label is what the user called the layout.
CREATE TABLE statement_profiles (
    user_id TEXT NOT NULL,
    signature TEXT NOT NULL,
    mapping_json TEXT NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, signature),
    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);
