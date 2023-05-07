#!/bin/bash

function merge_fonts() {
	local font1="$1"
	local font2="$2"
	local font_out="$3"
	fontforge -lang=ff -script mergefonts.ff "$font1" "$font2" 2816 "$font_out"
}

# Populate fonts_reg array with all Regular .ttf and .otf files
mapfile -t fonts_reg < <(find . -type f \( -iname "*regular.ttf" -o -iname "*regular.otf" \) -printf "%p\n" | sort)

# Populate fonts_bold array with Bold .ttf and .otf files or fallback to Regular
fonts_bold=()
for font in "${fonts_reg[@]}"; do
	bold_font="${font/regular/bold}"
	if [[ -f $bold_font ]]; then
		fonts_bold+=("$bold_font")
	else
		fonts_bold+=("$font")
	fi
done

# Print the list of fonts to be merged for each variation
echo "Fonts to be merged for Regular:"
printf "  %s\n" "${fonts_reg[@]}"
echo ""

echo "Fonts to be merged for Bold:"
printf "  %s\n" "${fonts_bold[@]}"
echo ""

# Ask for user confirmation
echo "Do you want to proceed with merging these fonts? (y/n)"
read -r response

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

for i in $(seq 1 $((${#fonts_reg[@]} - 2))); do
	rm "reg${i}.ttf"
done

for i in $(seq 1 $((${#fonts_bold[@]} - 2))); do
	rm "bold${i}.ttf"
done

rm 1.ttf
rm 2.ttf
