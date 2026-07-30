#!/bin/bash
# Menjalankan DevTray menggunakan System Python untuk kompatibilitas PyGObject / GTK
export PYTHONPATH=src
echo "Memulai DevTray..."
/usr/bin/python3 main.py
