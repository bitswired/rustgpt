#!/bin/bash

OUTPUT_FILE="all_files_aggregated.txt"
> "$OUTPUT_FILE"  # Create or clear the output file

# Loop over all files that are not listed in .gitignore
while IFS= read -r file; do
    echo "[PATH: $file]" >> "$OUTPUT_FILE"
    cat "$file" >> "$OUTPUT_FILE"
    echo -e "\n" >> "$OUTPUT_FILE"
done < <(git ls-files)

echo "Aggregation complete. Output in $OUTPUT_FILE"
