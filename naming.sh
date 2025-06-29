#!/bin/bash
# Script kiểm tra định danh cho tất cả các crates trong workspace
# Chạy từ thư mục gốc workspace
set -e

for crate in crates/*; do
  if [ -d "$crate/src" ]; then
    echo "==> Đang kiểm tra: $crate"
    (cd "$crate" && ../../naming src &> naming_report.txt)
  fi
done 