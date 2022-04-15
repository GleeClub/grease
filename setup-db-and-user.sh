#/bin/sh

echo "CREATE DATABASE IF NOT EXISTS grease;" | mysql
echo "CREATE USER 'grease'@'localhost' IDENTIFIED BY 'grease';" | mysql
echo "GRANT ALL PRIVILEGES ON * . * TO 'grease'@'localhost';" | mysql
