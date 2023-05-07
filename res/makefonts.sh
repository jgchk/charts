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
	rm 1.otf
	rm 2.otf
}

fonts_reg=(
	"inter/regular.otf"
	"noto-arabic/regular.ttf"
	"noto-bengali/regular.ttf"
	# "noto-chinese-simplified/regular.otf"
	# "noto-chinese-traditional/regular.otf"
	"noto-ethiopic/regular.ttf"
	"noto-emoji/regular.ttf"
	"noto-hebrew/regular.ttf"
	"noto-hindi/regular.ttf"
	"noto-japanese/regular.ttf"
	"noto-korean/regular.otf"
	"noto-thai/regular.ttf"
)

fonts_bold=(
	"inter/bold.otf"
	"noto-arabic/bold.ttf"
	"noto-bengali/bold.ttf"
	# "noto-chinese-simplified/bold.otf"
	# "noto-chinese-traditional/bold.otf"
	"noto-ethiopic/bold.ttf"
	"noto-emoji/bold.ttf"
	"noto-hebrew/bold.ttf"
	"noto-hindi/bold.ttf"
	"noto-japanese/bold.ttf"
	"noto-korean/bold.otf"
	"noto-thai/bold.ttf"
)

# Merge regular fonts
output="reg"
input="${fonts_reg[0]}"
for i in "${!fonts_reg[@]}"; do
	if [ $i -eq 0 ]; then
		continue
	fi
	output="reg${i}.otf"
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
	output="bold${i}.otf"
	merge_fonts "$input" "${fonts_bold[$i]}" "$output"
	input="$output"
done
mv "$output" bold_final.ttf

# Delete temporary files
for i in $(seq 1 $((${#fonts_reg[@]} - 2))); do
	rm "reg${i}.otf"
done
for i in $(seq 1 $((${#fonts_bold[@]} - 2))); do
	rm "bold${i}.otf"
done
