CREATE TABLE likes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) NOT NULL,
    post_id INTEGER REFERENCES posts(id) NOT NULL,
    UNIQUE(user_id, post_id),
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL  DEFAULT current_timestamp
);
