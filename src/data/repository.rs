use std::sync::Arc;

use sqlx::sqlite::SqlitePool;
use sqlx::{Row, Sqlite, Transaction};

use super::model::{Chat, ChatMessagePair};

#[derive(Clone)]
pub struct ChatRepository {
    pub pool: Arc<SqlitePool>,
}

impl ChatRepository {
    pub async fn get_all_chats(&self, user_id: i64) -> sqlx::Result<Vec<Chat>> {
        sqlx::query_as!(
            Chat,
            "SELECT id, user_id, name FROM chats WHERE user_id = ? ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn delete_chat(&self, chat_id: i64) -> sqlx::Result<u64> {
        let rows_affected = sqlx::query!("DELETE FROM chats WHERE id = ?", chat_id)
            .execute(&*self.pool)
            .await
            .unwrap()
            .rows_affected();
        Ok(rows_affected)
    }

    pub async fn retrieve_chat(&self, chat_id: i64) -> sqlx::Result<Vec<ChatMessagePair>> {
        sqlx::query_as!(
            ChatMessagePair,
            "SELECT * FROM v_chat_messages WHERE chat_id = ?",
            chat_id
        )
        .fetch_all(&*self.pool)
        .await
    }
    pub async fn create_chat(&self, user_id: i64, name: &str, model: &str) -> sqlx::Result<i64> {
        //create chat
        let chat = sqlx::query!(
            r#"
            INSERT INTO chats (user_id, name, model)
            VALUES (?, ?, ?) RETURNING id;
            "#,
            user_id,
            name,
            model
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(chat.id)
    }
    pub async fn add_ai_message_to_pair(&self, pair_id: i64, message: &str) -> sqlx::Result<i64> {
        let mut tx: Transaction<Sqlite> = self.pool.begin().await?;

        let message = sqlx::query!(
            r#"
            INSERT INTO messages (message)
            VALUES (?) RETURNING id;
            "#,
            message
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            UPDATE message_pairs
            SET ai_message_id = ?
            WHERE id = ?;
            "#,
            message.id,
            pair_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(message.id)
    }

    pub async fn add_message_block(&self, chat_id: i64, human_message: &str) -> sqlx::Result<i64> {
        //create chat
        let mut tx: Transaction<Sqlite> = self.pool.begin().await?;

        let message_block = sqlx::query!(
            r#"
            INSERT INTO message_blocks (chat_id)
            VALUES (?) RETURNING id;
            "#,
            chat_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        let message = sqlx::query!(
            r#"
            INSERT INTO messages (message)
            VALUES (?) RETURNING id;
            "#,
            human_message
        )
        .fetch_one(&mut *tx)
        .await?;

        let message_pair = sqlx::query!(
            r#"
            INSERT INTO message_pairs (human_message_id, message_block_id)
            VALUES (?, ?) RETURNING id;
            "#,
            message.id,
            message_block.id,
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            UPDATE message_blocks
            SET selected_pair_id = ?
            WHERE id = ?;
            "#,
            message_pair.id,
            message_block.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(message_pair.id)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use sqlx::migrate::Migrator;

    use super::*;

    async fn setup() -> (Arc<SqlitePool>, ChatRepository, i64) {
        let x = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:db.db".to_string());
        let pool = SqlitePool::connect(&x).await.unwrap();

        // let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let pool = Arc::new(pool);

        let migrator = Migrator::new(Path::new(dotenv::var("MIGRATIONS_PATH").unwrap().as_str()))
            .await
            .unwrap();
        // Run the migrations.
        migrator.run(&*pool).await.unwrap();

        let user = sqlx::query!(
            "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id",
            "test@test.com",
            "test"
        )
        .fetch_one(&*pool)
        .await
        .unwrap();

        let repo = ChatRepository { pool: pool.clone() };

        (pool, repo, user.id)
    }

    #[tokio::test]
    async fn test_create_chat() {
        let (pool, repo, user_id) = setup().await;
        let chat = repo.create_chat(user_id, "test", "gpt-4").await;
        assert!(chat.is_ok(), "Failed to create chat");
    }

    #[tokio::test]
    async fn test_add_message_block() {
        let (pool, repo, user_id) = setup().await;
        let chat = repo.create_chat(user_id, "test", "gpt-4").await;
        assert!(chat.is_ok(), "Failed to create chat");
        let chat_id = chat.unwrap();

        let message_block = repo.add_message_block(chat_id, "Test").await;
        assert!(message_block.is_ok(), "Failed to add message_block")
    }

    #[tokio::test]
    async fn test_json() {
        let (pool, repo, user_id) = setup().await;
        let chat = repo.create_chat(user_id, "test", "gpt-4").await;
        assert!(chat.is_ok(), "Failed to create chat");
        let chat_id = chat.unwrap();

        let message_block = repo.add_message_block(chat_id, "Test").await;
        assert!(message_block.is_ok(), "Failed to add message_block");

        let chat_message_pairs = repo.retrieve_chat(chat_id).await;
        print!("{:#?}", chat_message_pairs)
    }
}
