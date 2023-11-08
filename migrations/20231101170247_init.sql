CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT NOT NULL,
  password TEXT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE settings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL UNIQUE,
  openai_api_key TEXT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE messages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  message TEXT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE message_blocks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  chat_id INTEGER NOT NULL,
  selected_pair_id INTEGER,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
);

CREATE TABLE chats (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  model TEXT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE message_pairs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  human_message_id INTEGER NOT NULL,
  ai_message_id INTEGER,
  message_block_id INTEGER NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (human_message_id) REFERENCES messages(id) ON DELETE CASCADE,
  FOREIGN KEY (ai_message_id) REFERENCES messages(id) ON DELETE CASCADE,
  FOREIGN KEY (message_block_id) REFERENCES message_blocks(id) ON DELETE CASCADE
);

-- CREATE TRIGGER update_message_block_timestamp
-- AFTER
-- UPDATE
--   ON message_blocks FOR EACH ROW BEGIN
-- UPDATE
--   message_blocks
-- SET
--   updated_at = CURRENT_TIMESTAMP
-- WHERE
--   id = old.id;
-- END;
CREATE VIEW v_chat_messages AS
SELECT
  message_pairs.id,
  message_block_id,
  message_blocks.chat_id AS chat_id,
  chats.model AS model,
  human_message.message AS human_message,
  ai_message.message AS ai_message,
  RANK() OVER (
    PARTITION BY message_block_id
    ORDER BY
      message_pairs.created_at ASC
  ) AS block_rank,
  COUNT(*) OVER (PARTITION BY message_block_id) AS block_size
FROM
  message_pairs
  JOIN messages human_message ON human_message.id = message_pairs.human_message_id
  LEFT JOIN messages ai_message ON ai_message.id = message_pairs.ai_message_id
  JOIN message_blocks ON message_blocks.id = message_pairs.message_block_id
  JOIN chats ON chats.id = message_blocks.chat_id
WHERE
  message_pairs.id = message_blocks.selected_pair_id
ORDER BY
  message_blocks.created_at ASC;