#!/bin/bash

function merge_fonts() {
	local font1="$1"
	local font2="$2"
	local font_out="$3"

	echo ""
	echo ""
	echo "Merging $font1 and $font2 to $font_out"
	echo ""

	fontforge -lang=ff -script mergefonts.ff "$font1" "$font2" 2816 "$font_out"
	rm 1.ttf
	rm 2.ttf
}

fonts_reg=(
	"inter/regular.ttf"
	"noto-cjk/japanese/regular.otf"
	"noto-cjk/korean/regular.otf"
	"noto-cjk/simplified-chinese/regular.otf"
	"noto-cjk/traditional-chinese/regular.otf"
	"noto-ethiopic/regular.ttf"
	"noto-emoji/regular.ttf"
)

fonts_bold=(
	"inter/bold.ttf"
	"noto-cjk/japanese/bold.otf"
	"noto-cjk/korean/bold.otf"
	"noto-cjk/simplified-chinese/bold.otf"
	"noto-cjk/traditional-chinese/bold.otf"
	"noto-ethiopic/bold.ttf"
	"noto-emoji/bold.ttf"
)

# Merge regular fonts
output="reg"
input="${fonts_reg[0]}"
for i in "${!fonts_reg[@]}"; do
	if [ $i -eq 0 ]; then
		continue
	fi
	output="reg${i}.ttf"
	merge_fonts "$input" "${fonts_reg[$i]}" "$output"
	input="$output"
done
mv "$output" reg_final.ttf

# Merge bold fonts
output="bold"
input="${fonts_bold[0]}"
for i in "${!fonts_bold[@]}"; do
	if [ $i -eq 0 ]; then
		continue
	fi
	output="bold${i}.ttf"
	merge_fonts "$input" "${fonts_bold[$i]}" "$output"
	input="$output"
done
mv "$output" bold_final.ttf

# Delete temporary files
for i in $(seq 1 $((${#fonts_reg[@]} - 2))); do
	rm "reg${i}.ttf"
done
for i in $(seq 1 $((${#fonts_bold[@]} - 2))); do
	rm "bold${i}.ttf"
done
