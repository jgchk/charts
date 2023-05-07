#!/bin/bash

function merge_fonts() {
	local em_size="$1"
	local font_out="$2"
	shift 2
	fontforge -lang=ff -script mergefonts.ff "$em_size" "$font_out" "$@"
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

if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
	em_size=2816
	merge_fonts $em_size reg_final.ttf "${fonts_reg[@]}"
	merge_fonts $em_size bold_final.ttf "${fonts_bold[@]}"
else
	echo "Aborting..."
fi
