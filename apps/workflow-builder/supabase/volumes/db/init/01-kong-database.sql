-- Kong API Gateway Database Setup
-- Creates a separate database and user for Kong

-- Create Kong database
CREATE DATABASE kong;

-- Create Kong user with password
CREATE USER kong WITH PASSWORD 'kong';

-- Grant all privileges on Kong database to Kong user
GRANT ALL PRIVILEGES ON DATABASE kong TO kong;

-- Connect to Kong database and grant schema privileges
\c kong
GRANT ALL ON SCHEMA public TO kong;
