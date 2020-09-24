#!/bin/bash
sqlite3 db ".read import.sql"
sqlite3 db ".read init_tables.sql"
