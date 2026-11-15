// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! H.264 of the test videos, written out bit by bit: gray pictures in the
//! Constrained Baseline profile, whose sync pictures are IDR pictures that
//! hold their pixels as they are, in I_PCM macroblocks, and whose other
//! pictures are P pictures that make the picture before them a step
//! lighter
//!
//! A P picture predicts each macroblock from the same place in the picture
//! before it, and adds one coefficient of 3 to each block of four by four
//! pixels, which at QP 24 makes every pixel 8 lighter, with deblocking off
//! so that every pixel of a picture keeps the same shade.  Decoding a P
//! picture thus needs the pictures before it back to the last sync
//! picture, as with any encoder.  Pictures show in the order they decode,
//! which the parameter sets derive from the numbers of the frames.

/// Width and height of a picture, in macroblocks of 16 by 16 pixels
pub(super) const MACROBLOCKS: (usize, usize) = (4, 3);

/// Step of luma that each P picture adds to the picture before it
pub(super) const STEP: usize = 8;

/// Quantization parameter of every picture, at which a coefficient of 3
/// makes [`STEP`]
const QP: i32 = 24;

/// Bits of the syntax of H.264, most significant first, which each method
/// adds to
#[derive(Default)]
struct Bits(Vec<bool>);

impl Bits {
    /// Adds the `count` lowest bits of `value`
    fn u(self, count: u32, value: u32) -> Self {
        let Self(mut bits) = self;
        bits.extend((0..count).rev().map(|bit| value >> bit & 1 == 1));
        Self(bits)
    }

    /// Adds `value` as an unsigned Exp-Golomb code
    fn ue(self, value: u32) -> Self {
        let code = value + 1;
        let length = u32::BITS - code.leading_zeros();
        self.u(length - 1, 0).u(length, code)
    }

    /// Adds `value` as a signed Exp-Golomb code
    fn se(self, value: i32) -> Self {
        let magnitude = value.unsigned_abs();
        self.ue(if value > 0 {
            2 * magnitude - 1
        } else {
            2 * magnitude
        })
    }

    /// Adds zero bits up to the start of the next byte
    fn aligned(self) -> Self {
        let pad = (8 - self.0.len() % 8) % 8;
        self.u(pad as u32, 0)
    }

    fn bytes(self, count: usize, value: u8) -> Self {
        (0..count).fold(self, |bits, _| bits.u(8, u32::from(value)))
    }

    /// Returns the bits as the payload of a NAL unit of type `kind` and
    /// `nal_ref_idc` `reference`, after the stop bit and the zero bits that
    /// end a payload, and with a byte of 3 after each two zero bytes that a
    /// byte of 3 or less follows, so that no part of it reads as a start
    /// code
    fn unit(self, reference: u8, kind: u8) -> Vec<u8> {
        let Self(bits) = self.u(1, 1).aligned();
        bits.chunks(8)
            .map(|byte| byte.iter().fold(0, |byte, &bit| byte << 1 | u8::from(bit)))
            .fold(vec![reference << 5 | kind], |mut unit, byte| {
                if unit.ends_with(&[0, 0]) && byte <= 3 {
                    unit.push(3);
                }
                unit.push(byte);
                unit
            })
    }
}

pub(super) fn sequence_parameter_set() -> Vec<u8> {
    let (width, height) = MACROBLOCKS;
    Bits::default()
        .u(8, 66) // profile_idc of Baseline
        .u(8, 0b1100_0000) // constraint_set0_flag and constraint_set1_flag of Constrained Baseline
        .u(8, 10) // level_idc of level 1
        .ue(0) // seq_parameter_set_id
        .ue(0) // log2_max_frame_num_minus4, for frame_num modulo 16
        .ue(2) // pic_order_cnt_type of pictures that show in the order they decode
        .ue(1) // max_num_ref_frames
        .u(1, 0) // gaps_in_frame_num_value_allowed_flag
        .ue(width as u32 - 1) // pic_width_in_mbs_minus1
        .ue(height as u32 - 1) // pic_height_in_map_units_minus1
        .u(1, 1) // frame_mbs_only_flag
        .u(1, 1) // direct_8x8_inference_flag
        .u(1, 0) // frame_cropping_flag
        .u(1, 0) // vui_parameters_present_flag
        .unit(3, 7)
}

pub(super) fn picture_parameter_set() -> Vec<u8> {
    Bits::default()
        .ue(0) // pic_parameter_set_id
        .ue(0) // seq_parameter_set_id
        .u(1, 0) // entropy_coding_mode_flag of CAVLC
        .u(1, 0) // bottom_field_pic_order_in_frame_present_flag
        .ue(0) // num_slice_groups_minus1
        .ue(0) // num_ref_idx_l0_default_active_minus1
        .ue(0) // num_ref_idx_l1_default_active_minus1
        .u(1, 0) // weighted_pred_flag
        .u(2, 0) // weighted_bipred_idc
        .se(QP - 26) // pic_init_qp_minus26
        .se(0) // pic_init_qs_minus26
        .se(0) // chroma_qp_index_offset
        .u(1, 1) // deblocking_filter_control_present_flag, so that slices turn deblocking off
        .u(1, 0) // constrained_intra_pred_flag
        .u(1, 0) // redundant_pic_cnt_present_flag
        .unit(3, 8)
}

/// Returns the NAL unit of picture `number` of a test video whose sync
/// pictures come every `sync_every` pictures: at a sync picture an IDR
/// picture of luma `luma`, and otherwise a P picture a step lighter than
/// the picture before it, which leaves `luma` unused
pub(super) fn picture(number: usize, sync_every: usize, luma: u8) -> Vec<u8> {
    let (width, height) = MACROBLOCKS;
    let macroblocks = width * height;
    let since_sync = number % sync_every;
    if since_sync == 0 {
        let header = Bits::default()
            .ue(0) // first_mb_in_slice
            .ue(7) // slice_type of I, which every slice of the picture is
            .ue(0) // pic_parameter_set_id
            .u(4, 0) // frame_num
            .ue((number / sync_every % 2) as u32) // idr_pic_id, which IDR pictures in a row differ in
            .u(1, 0) // no_output_of_prior_pics_flag
            .u(1, 0) // long_term_reference_flag
            .se(0) // slice_qp_delta
            .ue(1); // disable_deblocking_filter_idc
        (0..macroblocks)
            .fold(header, |bits, _| {
                bits.ue(25) // mb_type of I_PCM
                    .aligned()
                    .bytes(256, luma)
                    .bytes(128, 128)
            })
            .unit(3, 5)
    } else {
        let header = Bits::default()
            .ue(0) // first_mb_in_slice
            .ue(5) // slice_type of P, which every slice of the picture is
            .ue(0) // pic_parameter_set_id
            .u(4, (since_sync % 16) as u32) // frame_num
            .u(1, 0) // num_ref_idx_active_override_flag
            .u(1, 0) // ref_pic_list_modification_flag_l0
            .u(1, 0) // adaptive_ref_pic_marking_mode_flag
            .se(0) // slice_qp_delta
            .ue(1); // disable_deblocking_filter_idc
        (0..macroblocks)
            .fold(header, |bits, _| lighter(bits))
            .unit(2, 1)
    }
}

/// Adds a macroblock of a P picture that it predicts from the same place in
/// the picture before it, and makes a step lighter
fn lighter(bits: Bits) -> Bits {
    let bits = bits
        .ue(0) // mb_skip_run
        .ue(0) // mb_type of P_L0_16x16
        .se(0) // mvd_l0 across
        .se(0) // mvd_l0 down
        .ue(11) // coded_block_pattern of every block of luma and no chroma, in an inter macroblock
        .se(0); // mb_qp_delta
    // Each block of four by four holds 3 at the place of DC alone: a
    // coeff_token of one coefficient and no trailing ones, in the table for
    // blocks with fewer than two coefficients around, a level_prefix of 2,
    // which makes 3 for the first coefficient after no trailing ones, and a
    // total_zeros of none
    (0..16).fold(bits, |bits, _| bits.u(6, 0b00_0101).u(3, 0b001).u(1, 1))
}
