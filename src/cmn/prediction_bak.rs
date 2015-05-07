

/* CORR_PRED():
	- first has to be a new prediction from the previous frame
	- then has to have come true

*/
pub fn corr_pred(
			out: u8, 
			ff: u8, 
			prev_out: u8, 
			prev_ff: u8, 
) -> bool {
	let prev_new_pred = new_pred(prev_out, prev_ff);

	if prev_new_pred && (ff != 0) {
		true
	} else {
		false
	}
}

pub fn new_pred(
			out: u8, 
			ff: u8, 
) -> bool {
	let out_active = out != 0;
	let ff_active = ff != 0;
	let pred = out != ff;
	let new_pred = pred && (!ff_active);

	new_pred
}
