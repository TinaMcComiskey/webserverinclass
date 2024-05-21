CREATE TABLE IF NOT EXISTS questions (
  id TEXT PRIMARY KEY,
  question TEXT NOT NULL,
  answer TEXT,
  source TEXT
);

CREATE TABLE IF NOT EXISTS tags (
  id TEXT REFERENCES jokes(id),
  tag TEXT NOT NULL
);