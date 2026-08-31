use core::arch::aarch64::*;

use crate::{
    BLOCK_LEN, CVWords, IV, IncrementCounter, MSG_SCHEDULE, OUT_LEN, counter_high, counter_low,
};

pub const DEGREE: usize = 4;

#[inline(always)]
unsafe fn loadu(src: *const u8) -> uint32x4_t {
    unsafe { vreinterpretq_u32_u8(vld1q_u8(src)) }
}

#[inline(always)]
unsafe fn storeu(src: uint32x4_t, dest: *mut u8) {
    unsafe { vst1q_u8(dest, vreinterpretq_u8_u32(src)) }
}

#[inline(always)]
unsafe fn add(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    unsafe { vaddq_u32(a, b) }
}

#[inline(always)]
unsafe fn xor(a: uint32x4_t, b: uint32x4_t) -> uint32x4_t {
    unsafe { veorq_u32(a, b) }
}

#[inline(always)]
unsafe fn set1(x: u32) -> uint32x4_t {
    unsafe { vdupq_n_u32(x) }
}

#[inline(always)]
unsafe fn set4(a: u32, b: u32, c: u32, d: u32) -> uint32x4_t {
    let words = [a, b, c, d];
    unsafe { vld1q_u32(words.as_ptr()) }
}

#[inline(always)]
unsafe fn rot16(x: uint32x4_t) -> uint32x4_t {
    unsafe { vreinterpretq_u32_u16(vrev32q_u16(vreinterpretq_u16_u32(x))) }
}

#[inline(always)]
unsafe fn rot12(x: uint32x4_t) -> uint32x4_t {
    unsafe { vsriq_n_u32::<12>(vshlq_n_u32::<20>(x), x) }
}

#[inline(always)]
unsafe fn rot8(x: uint32x4_t) -> uint32x4_t {
    let indices = [1, 2, 3, 0, 5, 6, 7, 4, 9, 10, 11, 8, 13, 14, 15, 12];
    unsafe {
        vreinterpretq_u32_u8(vqtbl1q_u8(
            vreinterpretq_u8_u32(x),
            vld1q_u8(indices.as_ptr()),
        ))
    }
}

#[inline(always)]
unsafe fn rot7(x: uint32x4_t) -> uint32x4_t {
    unsafe { vsriq_n_u32::<7>(vshlq_n_u32::<25>(x), x) }
}

#[inline(always)]
unsafe fn round(v: &mut [uint32x4_t; 16], m: &[uint32x4_t; 16], r: usize) {
    unsafe {
        v[0] = add(v[0], m[MSG_SCHEDULE[r][0]]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][2]]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][4]]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][6]]);
        v[0] = add(v[0], v[4]);
        v[1] = add(v[1], v[5]);
        v[2] = add(v[2], v[6]);
        v[3] = add(v[3], v[7]);
        v[12] = rot16(xor(v[12], v[0]));
        v[13] = rot16(xor(v[13], v[1]));
        v[14] = rot16(xor(v[14], v[2]));
        v[15] = rot16(xor(v[15], v[3]));
        v[8] = add(v[8], v[12]);
        v[9] = add(v[9], v[13]);
        v[10] = add(v[10], v[14]);
        v[11] = add(v[11], v[15]);
        v[4] = rot12(xor(v[4], v[8]));
        v[5] = rot12(xor(v[5], v[9]));
        v[6] = rot12(xor(v[6], v[10]));
        v[7] = rot12(xor(v[7], v[11]));

        v[0] = add(v[0], m[MSG_SCHEDULE[r][1]]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][3]]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][5]]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][7]]);
        v[0] = add(v[0], v[4]);
        v[1] = add(v[1], v[5]);
        v[2] = add(v[2], v[6]);
        v[3] = add(v[3], v[7]);
        v[12] = rot8(xor(v[12], v[0]));
        v[13] = rot8(xor(v[13], v[1]));
        v[14] = rot8(xor(v[14], v[2]));
        v[15] = rot8(xor(v[15], v[3]));
        v[8] = add(v[8], v[12]);
        v[9] = add(v[9], v[13]);
        v[10] = add(v[10], v[14]);
        v[11] = add(v[11], v[15]);
        v[4] = rot7(xor(v[4], v[8]));
        v[5] = rot7(xor(v[5], v[9]));
        v[6] = rot7(xor(v[6], v[10]));
        v[7] = rot7(xor(v[7], v[11]));

        v[0] = add(v[0], m[MSG_SCHEDULE[r][8]]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][10]]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][12]]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][14]]);
        v[0] = add(v[0], v[5]);
        v[1] = add(v[1], v[6]);
        v[2] = add(v[2], v[7]);
        v[3] = add(v[3], v[4]);
        v[15] = rot16(xor(v[15], v[0]));
        v[12] = rot16(xor(v[12], v[1]));
        v[13] = rot16(xor(v[13], v[2]));
        v[14] = rot16(xor(v[14], v[3]));
        v[10] = add(v[10], v[15]);
        v[11] = add(v[11], v[12]);
        v[8] = add(v[8], v[13]);
        v[9] = add(v[9], v[14]);
        v[5] = rot12(xor(v[5], v[10]));
        v[6] = rot12(xor(v[6], v[11]));
        v[7] = rot12(xor(v[7], v[8]));
        v[4] = rot12(xor(v[4], v[9]));

        v[0] = add(v[0], m[MSG_SCHEDULE[r][9]]);
        v[1] = add(v[1], m[MSG_SCHEDULE[r][11]]);
        v[2] = add(v[2], m[MSG_SCHEDULE[r][13]]);
        v[3] = add(v[3], m[MSG_SCHEDULE[r][15]]);
        v[0] = add(v[0], v[5]);
        v[1] = add(v[1], v[6]);
        v[2] = add(v[2], v[7]);
        v[3] = add(v[3], v[4]);
        v[15] = rot8(xor(v[15], v[0]));
        v[12] = rot8(xor(v[12], v[1]));
        v[13] = rot8(xor(v[13], v[2]));
        v[14] = rot8(xor(v[14], v[3]));
        v[10] = add(v[10], v[15]);
        v[11] = add(v[11], v[12]);
        v[8] = add(v[8], v[13]);
        v[9] = add(v[9], v[14]);
        v[5] = rot7(xor(v[5], v[10]));
        v[6] = rot7(xor(v[6], v[11]));
        v[7] = rot7(xor(v[7], v[8]));
        v[4] = rot7(xor(v[4], v[9]));
    }
}

#[inline(always)]
unsafe fn transpose_vecs(vecs: &mut [uint32x4_t; DEGREE]) {
    unsafe {
        let rows01 = vtrnq_u32(vecs[0], vecs[1]);
        let rows23 = vtrnq_u32(vecs[2], vecs[3]);
        vecs[0] = vcombine_u32(vget_low_u32(rows01.0), vget_low_u32(rows23.0));
        vecs[1] = vcombine_u32(vget_low_u32(rows01.1), vget_low_u32(rows23.1));
        vecs[2] = vcombine_u32(vget_high_u32(rows01.0), vget_high_u32(rows23.0));
        vecs[3] = vcombine_u32(vget_high_u32(rows01.1), vget_high_u32(rows23.1));
    }
}

#[inline(always)]
unsafe fn transpose_msg_vecs(
    inputs: &[*const u8; DEGREE],
    block_offset: usize,
) -> [uint32x4_t; 16] {
    unsafe {
        let mut out = [
            loadu(inputs[0].add(block_offset)),
            loadu(inputs[1].add(block_offset)),
            loadu(inputs[2].add(block_offset)),
            loadu(inputs[3].add(block_offset)),
            loadu(inputs[0].add(block_offset + 16)),
            loadu(inputs[1].add(block_offset + 16)),
            loadu(inputs[2].add(block_offset + 16)),
            loadu(inputs[3].add(block_offset + 16)),
            loadu(inputs[0].add(block_offset + 32)),
            loadu(inputs[1].add(block_offset + 32)),
            loadu(inputs[2].add(block_offset + 32)),
            loadu(inputs[3].add(block_offset + 32)),
            loadu(inputs[0].add(block_offset + 48)),
            loadu(inputs[1].add(block_offset + 48)),
            loadu(inputs[2].add(block_offset + 48)),
            loadu(inputs[3].add(block_offset + 48)),
        ];
        for square in out.chunks_exact_mut(DEGREE) {
            transpose_vecs(square.try_into().unwrap());
        }
        out
    }
}

#[inline(always)]
unsafe fn load_counters(
    counter: u64,
    increment_counter: IncrementCounter,
) -> (uint32x4_t, uint32x4_t) {
    let mask = if increment_counter.yes() { !0 } else { 0 };
    unsafe {
        (
            set4(
                counter_low(counter),
                counter_low(counter + (mask & 1)),
                counter_low(counter + (mask & 2)),
                counter_low(counter + (mask & 3)),
            ),
            set4(
                counter_high(counter),
                counter_high(counter + (mask & 1)),
                counter_high(counter + (mask & 2)),
                counter_high(counter + (mask & 3)),
            ),
        )
    }
}

#[target_feature(enable = "neon")]
unsafe fn hash4(
    inputs: &[*const u8; DEGREE],
    blocks: usize,
    key: &CVWords,
    counter: u64,
    increment_counter: IncrementCounter,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    out: &mut [u8; DEGREE * OUT_LEN],
) {
    unsafe {
        let mut h_vecs = [
            set1(key[0]),
            set1(key[1]),
            set1(key[2]),
            set1(key[3]),
            set1(key[4]),
            set1(key[5]),
            set1(key[6]),
            set1(key[7]),
        ];
        let (counter_low_vec, counter_high_vec) = load_counters(counter, increment_counter);
        let mut block_flags = flags | flags_start;

        for block in 0..blocks {
            if block + 1 == blocks {
                block_flags |= flags_end;
            }
            let msg_vecs = transpose_msg_vecs(inputs, block * BLOCK_LEN);
            let mut v = [
                h_vecs[0],
                h_vecs[1],
                h_vecs[2],
                h_vecs[3],
                h_vecs[4],
                h_vecs[5],
                h_vecs[6],
                h_vecs[7],
                set1(IV[0]),
                set1(IV[1]),
                set1(IV[2]),
                set1(IV[3]),
                counter_low_vec,
                counter_high_vec,
                set1(BLOCK_LEN as u32),
                set1(block_flags as u32),
            ];
            round(&mut v, &msg_vecs, 0);
            round(&mut v, &msg_vecs, 1);
            round(&mut v, &msg_vecs, 2);
            round(&mut v, &msg_vecs, 3);
            round(&mut v, &msg_vecs, 4);
            round(&mut v, &msg_vecs, 5);
            round(&mut v, &msg_vecs, 6);
            for i in 0..8 {
                h_vecs[i] = xor(v[i], v[i + 8]);
            }
            block_flags = flags;
        }

        let (low, high) = h_vecs.split_at_mut(DEGREE);
        transpose_vecs(low.try_into().unwrap());
        transpose_vecs(high.try_into().unwrap());
        storeu(h_vecs[0], out.as_mut_ptr().add(0));
        storeu(h_vecs[4], out.as_mut_ptr().add(16));
        storeu(h_vecs[1], out.as_mut_ptr().add(32));
        storeu(h_vecs[5], out.as_mut_ptr().add(48));
        storeu(h_vecs[2], out.as_mut_ptr().add(64));
        storeu(h_vecs[6], out.as_mut_ptr().add(80));
        storeu(h_vecs[3], out.as_mut_ptr().add(96));
        storeu(h_vecs[7], out.as_mut_ptr().add(112));
    }
}

#[target_feature(enable = "neon")]
pub unsafe fn hash_many<const N: usize>(
    mut inputs: &[&[u8; N]],
    key: &CVWords,
    mut counter: u64,
    increment_counter: IncrementCounter,
    flags: u8,
    flags_start: u8,
    flags_end: u8,
    mut out: &mut [u8],
) {
    assert!(out.len() >= inputs.len() * OUT_LEN);
    while inputs.len() >= DEGREE {
        let input_ptrs: &[*const u8; DEGREE] =
            unsafe { &*(inputs.as_ptr() as *const [*const u8; DEGREE]) };
        unsafe {
            hash4(
                input_ptrs,
                N / BLOCK_LEN,
                key,
                counter,
                increment_counter,
                flags,
                flags_start,
                flags_end,
                (&mut out[..DEGREE * OUT_LEN]).try_into().unwrap(),
            );
        }
        if increment_counter.yes() {
            counter += DEGREE as u64;
        }
        inputs = &inputs[DEGREE..];
        out = &mut out[DEGREE * OUT_LEN..];
    }
    crate::portable::hash_many(
        inputs,
        key,
        counter,
        increment_counter,
        flags,
        flags_start,
        flags_end,
        out,
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transpose() {
        #[target_feature(enable = "neon")]
        unsafe fn transpose_wrapper(vecs: &mut [uint32x4_t; DEGREE]) {
            unsafe { transpose_vecs(vecs) };
        }

        let mut matrix = [[0u32; DEGREE]; DEGREE];
        for (i, row) in matrix.iter_mut().enumerate() {
            for (j, word) in row.iter_mut().enumerate() {
                *word = (i * DEGREE + j) as u32;
            }
        }
        unsafe {
            let mut vecs: [uint32x4_t; DEGREE] = core::mem::transmute(matrix);
            transpose_wrapper(&mut vecs);
            matrix = core::mem::transmute(vecs);
        }
        for i in 0..DEGREE {
            for j in 0..DEGREE {
                assert_eq!(matrix[j][i], (i * DEGREE + j) as u32);
            }
        }
    }

    #[test]
    fn test_hash_many() {
        crate::test::test_hash_many_fn(hash_many, hash_many);
    }
}
