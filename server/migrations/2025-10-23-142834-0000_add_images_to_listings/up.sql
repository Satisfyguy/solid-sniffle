-- Add images_ipfs_cids column to listings table
-- Stores an array of IPFS CIDs as JSON: ["Qm...", "Qm...", ...]
ALTER TABLE listings ADD COLUMN images_ipfs_cids TEXT DEFAULT '[]';
