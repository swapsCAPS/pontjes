#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
echo $DIR
DATA_DIR="$DIR/../data"
cd $DATA_DIR
echo "Importing..."
sqlite3 $DATA_DIR/pontjes_db ".read ../scripts/import.sql"
sqlite3 $DATA_DIR/pontjes_db ".read ../scripts/init_tables.sql"
echo "Done importing"