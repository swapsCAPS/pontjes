#!/bin/bash
sqlite3 data/pontjes_db ".read import.sql"
sqlite3 data/pontjes_db ".read init_tables.sql"
