#/bin/sh

echo "CREATE DATABASE IF NOT EXISTS grease;" > mysql
echo "CREATE USER 'grease'@'localhost' IDENTIFIED BY 'grease';" > mysql
echo "GRANT ALL PRIVILEGES ON * . * TO 'grease'@'localhost';" > mysql
mysql --database=grease --user=grease --password=grease < migrations/2018-08-22-214705_create_tables/up.sql