#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
echo $DIR
DATA_DIR="$DIR/../data"
echo $DATA_DIR
cd $DATA_DIR
echo "Importing..."
sqlite3 $DATA_DIR/tmp_pontjes_db ".read ../scripts/import.sql"
echo "Initializing tables..."
sqlite3 $DATA_DIR/tmp_pontjes_db ".read ../scripts/init_tables.sql"
echo moving $DATA_DIR/tmp_pontjes_db to $DATA_DIR/pontjes_db
mv $DATA_DIR/tmp_pontjes_db $DATA_DIR/pontjes_db
echo "Noting download date in $DATA_DIR/download_date file, reload pontjes to pick it up"
echo $(date +%Y%m%d) > $DATA_DIR/download_date
echo "Done! : )"
