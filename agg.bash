#!/bin/bash

OUTPUT_FILE="all_files_aggregated.txt"
> "$OUTPUT_FILE"  # Create or clear the output file

while IFS= read -r -d '' file; do
    # Skip binary files
    if [[ -f "$file" && ! -I "$file" ]]; then
        echo "[PATH: $file]" >> "$OUTPUT_FILE"
        cat "$file" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
done < <(git ls-files -z)

echo "Aggregation complete. Output in $OUTPUT_FILE"