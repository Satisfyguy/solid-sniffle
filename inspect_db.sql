PRAGMA table_info(listings);
SELECT sql FROM sqlite_master WHERE name='listings';
SELECT COUNT(*) FROM users WHERE role='vendor';
SELECT id, username, role FROM users LIMIT 3;
