pub static PAGES: [&str; 256] = [
	"unicode_page_00", 
	"unicode_page_01", 
	"unicode_page_02", 
	"unicode_page_03", 
	"unicode_page_04", 
	"unicode_page_05", 
	"unicode_page_06", 
	"unicode_page_07", 
	"unicode_page_08", 
	"unicode_page_09", 
	"unicode_page_0a", 
	"unicode_page_0b", 
	"unicode_page_0c", 
	"unicode_page_0d", 
	"unicode_page_0e", 
	"unicode_page_0f", 
	"unicode_page_10", 
	"unicode_page_11", 
	"unicode_page_12", 
	"unicode_page_13", 
	"unicode_page_14", 
	"unicode_page_15", 
	"unicode_page_16", 
	"unicode_page_17", 
	"unicode_page_18", 
	"unicode_page_19", 
	"unicode_page_1a", 
	"unicode_page_1b", 
	"unicode_page_1c", 
	"unicode_page_1d", 
	"unicode_page_1e", 
	"unicode_page_1f", 
	"unicode_page_20", 
	"unicode_page_21", 
	"unicode_page_22", 
	"unicode_page_23", 
	"unicode_page_24", 
	"unicode_page_25", 
	"unicode_page_26", 
	"unicode_page_27", 
	"unicode_page_28", 
	"unicode_page_29", 
	"unicode_page_2a", 
	"unicode_page_2b", 
	"unicode_page_2c", 
	"unicode_page_2d", 
	"unicode_page_2e", 
	"unicode_page_2f", 
	"unicode_page_30", 
	"unicode_page_31", 
	"unicode_page_32", 
	"unicode_page_33", 
	"unicode_page_34", 
	"unicode_page_35", 
	"unicode_page_36", 
	"unicode_page_37", 
	"unicode_page_38", 
	"unicode_page_39", 
	"unicode_page_3a", 
	"unicode_page_3b", 
	"unicode_page_3c", 
	"unicode_page_3d", 
	"unicode_page_3e", 
	"unicode_page_3f", 
	"unicode_page_40", 
	"unicode_page_41", 
	"unicode_page_42", 
	"unicode_page_43", 
	"unicode_page_44", 
	"unicode_page_45", 
	"unicode_page_46", 
	"unicode_page_47", 
	"unicode_page_48", 
	"unicode_page_49", 
	"unicode_page_4a", 
	"unicode_page_4b", 
	"unicode_page_4c", 
	"unicode_page_4d", 
	"unicode_page_4e", 
	"unicode_page_4f", 
	"unicode_page_50", 
	"unicode_page_51", 
	"unicode_page_52", 
	"unicode_page_53", 
	"unicode_page_54", 
	"unicode_page_55", 
	"unicode_page_56", 
	"unicode_page_57", 
	"unicode_page_58", 
	"unicode_page_59", 
	"unicode_page_5a", 
	"unicode_page_5b", 
	"unicode_page_5c", 
	"unicode_page_5d", 
	"unicode_page_5e", 
	"unicode_page_5f", 
	"unicode_page_60", 
	"unicode_page_61", 
	"unicode_page_62", 
	"unicode_page_63", 
	"unicode_page_64", 
	"unicode_page_65", 
	"unicode_page_66", 
	"unicode_page_67", 
	"unicode_page_68", 
	"unicode_page_69", 
	"unicode_page_6a", 
	"unicode_page_6b", 
	"unicode_page_6c", 
	"unicode_page_6d", 
	"unicode_page_6e", 
	"unicode_page_6f", 
	"unicode_page_70", 
	"unicode_page_71", 
	"unicode_page_72", 
	"unicode_page_73", 
	"unicode_page_74", 
	"unicode_page_75", 
	"unicode_page_76", 
	"unicode_page_77", 
	"unicode_page_78", 
	"unicode_page_79", 
	"unicode_page_7a", 
	"unicode_page_7b", 
	"unicode_page_7c", 
	"unicode_page_7d", 
	"unicode_page_7e", 
	"unicode_page_7f", 
	"unicode_page_80", 
	"unicode_page_81", 
	"unicode_page_82", 
	"unicode_page_83", 
	"unicode_page_84", 
	"unicode_page_85", 
	"unicode_page_86", 
	"unicode_page_87", 
	"unicode_page_88", 
	"unicode_page_89", 
	"unicode_page_8a", 
	"unicode_page_8b", 
	"unicode_page_8c", 
	"unicode_page_8d", 
	"unicode_page_8e", 
	"unicode_page_8f", 
	"unicode_page_90", 
	"unicode_page_91", 
	"unicode_page_92", 
	"unicode_page_93", 
	"unicode_page_94", 
	"unicode_page_95", 
	"unicode_page_96", 
	"unicode_page_97", 
	"unicode_page_98", 
	"unicode_page_99", 
	"unicode_page_9a", 
	"unicode_page_9b", 
	"unicode_page_9c", 
	"unicode_page_9d", 
	"unicode_page_9e", 
	"unicode_page_9f", 
	"unicode_page_a0", 
	"unicode_page_a1", 
	"unicode_page_a2", 
	"unicode_page_a3", 
	"unicode_page_a4", 
	"unicode_page_a5", 
	"unicode_page_a6", 
	"unicode_page_a7", 
	"unicode_page_a8", 
	"unicode_page_a9", 
	"unicode_page_aa", 
	"unicode_page_ab", 
	"unicode_page_ac", 
	"unicode_page_ad", 
	"unicode_page_ae", 
	"unicode_page_af", 
	"unicode_page_b0", 
	"unicode_page_b1", 
	"unicode_page_b2", 
	"unicode_page_b3", 
	"unicode_page_b4", 
	"unicode_page_b5", 
	"unicode_page_b6", 
	"unicode_page_b7", 
	"unicode_page_b8", 
	"unicode_page_b9", 
	"unicode_page_ba", 
	"unicode_page_bb", 
	"unicode_page_bc", 
	"unicode_page_bd", 
	"unicode_page_be", 
	"unicode_page_bf", 
	"unicode_page_c0", 
	"unicode_page_c1", 
	"unicode_page_c2", 
	"unicode_page_c3", 
	"unicode_page_c4", 
	"unicode_page_c5", 
	"unicode_page_c6", 
	"unicode_page_c7", 
	"unicode_page_c8", 
	"unicode_page_c9", 
	"unicode_page_ca", 
	"unicode_page_cb", 
	"unicode_page_cc", 
	"unicode_page_cd", 
	"unicode_page_ce", 
	"unicode_page_cf", 
	"unicode_page_d0", 
	"unicode_page_d1", 
	"unicode_page_d2", 
	"unicode_page_d3", 
	"unicode_page_d4", 
	"unicode_page_d5", 
	"unicode_page_d6", 
	"unicode_page_d7", 
	"unicode_page_d8", 
	"unicode_page_d9", 
	"unicode_page_da", 
	"unicode_page_db", 
	"unicode_page_dc", 
	"unicode_page_dd", 
	"unicode_page_de", 
	"unicode_page_df", 
	"unicode_page_e0", 
	"unicode_page_e1", 
	"unicode_page_e2", 
	"unicode_page_e3", 
	"unicode_page_e4", 
	"unicode_page_e5", 
	"unicode_page_e6", 
	"unicode_page_e7", 
	"unicode_page_e8", 
	"unicode_page_e9", 
	"unicode_page_ea", 
	"unicode_page_eb", 
	"unicode_page_ec", 
	"unicode_page_ed", 
	"unicode_page_ee", 
	"unicode_page_ef", 
	"unicode_page_f0", 
	"unicode_page_f1", 
	"unicode_page_f2", 
	"unicode_page_f3", 
	"unicode_page_f4", 
	"unicode_page_f5", 
	"unicode_page_f6", 
	"unicode_page_f7", 
	"unicode_page_f8", 
	"unicode_page_f9", 
	"unicode_page_fa", 
	"unicode_page_fb", 
	"unicode_page_fc", 
	"unicode_page_fd", 
	"unicode_page_fe", 
	"unicode_page_ff",
];

pub fn get_page(atlas: Option<u32>) -> &'static str {
	match atlas {
		Some(index) => PAGES[index as usize],
		None => "ascii"
	}
}