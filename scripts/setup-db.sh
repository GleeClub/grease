#!/bin/bash

# Create database and user for interacting locally

DB_NAME="grease"
DB_USERNAME="grease"
DB_PASSWORD="grease"

MYSQL_SCRIPT="
CREATE DATABASE IF NOT EXISTS $DB_NAME;
CREATE USER '$DB_USERNAME'@'localhost' IDENTIFIED BY '$DB_PASSWORD';
GRANT ALL PRIVILEGES ON * . * TO '$DB_USERNAME'@'localhost';"

echo "$MYSQL_SCRIPT" | mysql

# Add .env file for use with SQLx
echo "DATABASE_URL=mysql://$DB_USERNAME:$DB_PASSWORD@localhost/$DB_NAME" > .env
