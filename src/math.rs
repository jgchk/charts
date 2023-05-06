pub fn optimal_square(num_items: u32) -> (u32, u32) {
    let mut rows = (num_items as f64).sqrt().floor() as u32;
    let mut cols = rows;

    while rows * cols < num_items {
        if cols <= rows {
            cols += 1;
        } else {
            rows += 1;
        }
    }

    (rows, cols)
}
