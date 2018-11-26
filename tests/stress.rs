//! Stress tests adapted from:
//!     https://www.icir.org/vern/papers/testbase-report.pdf

extern crate lexical;
use std::fmt::Debug;


struct StressTest<T: 'static + lexical::FromBytes> {
    below_ulp: &'static [(&'static str, T)],
    above_ulp: &'static [(&'static str, T)],
}

struct StressTests {
    bit24: StressTest<f32>,
    bit56: StressTest<f64>,
}

const STRESS_TESTS: StressTests = StressTests {
    bit24: StressTest {
        below_ulp: &[
            ("5e-20", 5e-20),
            ("67e+14", 6700000000000000.0),
            ("985e+15", 9.85e+17),
            ("7693e-42", 7.693e-39),
            ("55895e-16", 5.5895e-12),
            ("996622e-44", 9.96622e-39),
            ("7038531e-32", 7.038531e-26),
            ("60419369e-46", 6.0419369e-39),
            ("702990899e-20", 7.02990899e-12),
            ("6930161142e-48", 6.930161142e-39),
            ("25933168707e+13", 2.5933168707e+23),
            ("596428896559e+20", 5.96428896559e+31),
        ],
        above_ulp: &[
            ("3e-23", 3e-23),
            ("57e+18", 5.7e+19),
            ("789e-35", 7.89e-33),
            ("2539e-18", 2.539e-15),
            ("76173e+28", 7.6173e+32),
            ("887745e-11", 8.87745e-06),
            ("5382571e-37", 5.382571e-31),
            ("82381273e-35", 8.2381273e-28),
            ("750486563e-38", 7.50486563e-30),
            ("3752432815e-39", 3.752432815e-30),
            ("75224575729e-45", 7.5224575729e-35),
            ("459926601011e+15", 4.59926601011e+26),
        ],
    },
    bit56: StressTest {
        below_ulp: &[
            ("7e-27", 7e-27),
            ("37e-29", 3.7e-28),
            ("743e-18", 7.43e-16),
            ("7861e-33", 7.861e-30),
            ("46073e-30", 4.6073e-26),
            ("774497e-34", 7.74497e-29),
            ("8184513e-33", 8.184513e-27),
            ("89842219e-28", 8.9842219e-21),
            ("449211095e-29", 4.49211095e-21),
            ("8128913627e-40", 8.128913627e-31),
            ("87365670181e-18", 8.7365670181e-08),
            ("436828350905e-19", 4.36828350905e-08),
            ("5569902441849e-49", 5.569902441849e-37),
            ("60101945175297e-32", 6.0101945175297e-19),
            ("754205928904091e-51", 7.54205928904091e-37),
            ("5930988018823113e-37", 5.930988018823113e-22),
            ("51417459976130695e-27", 5.14174599761307e-11),
            ("826224659167966417e-41", 8.262246591679664e-24),
            ("9612793100620708287e-57", 9.612793100620709e-39),
            ("93219542812847969081e-39", 9.321954281284797e-20),
            ("544579064588249633923e-48", 5.445790645882496e-28),
            ("4985301935905831716201e-48", 4.9853019359058315e-27),
        ],
        above_ulp: &[
            ("9e+26", 9e+26),
            ("79e-8", 7.9e-07),
            ("393e+26", 3.93e+28),
            ("9171e-40", 9.171e-37),
            ("56257e-16", 5.6257e-12),
            ("281285e-17", 2.81285e-12),
            ("4691113e-43", 4.691113e-37),
            ("29994057e-15", 2.9994057e-08),
            ("834548641e-46", 8.34548641e-38),
            ("1058695771e-47", 1.058695771e-38),
            ("87365670181e-18", 8.7365670181e-08),
            ("872580695561e-36", 8.72580695561e-25),
            ("6638060417081e-51", 6.638060417081e-39),
            ("88473759402752e-52", 8.8473759402752e-39),
            ("412413848938563e-27", 4.12413848938563e-13),
            ("5592117679628511e-48", 5.592117679628511e-33),
            ("83881765194427665e-50", 8.388176519442766e-34),
            ("638632866154697279e-35", 6.3863286615469725e-18),
            ("3624461315401357483e-53", 3.6244613154013577e-35),
            ("75831386216699428651e-30", 7.583138621669942e-11),
            ("356645068918103229683e-42", 3.5664506891810324e-22),
            ("7022835002724438581513e-33", 7.0228350027244384e-12),
        ],
    },
};

fn stress_test<T: 'static + Debug + PartialEq + lexical::FromBytes>(test: &StressTest<T>) {
    for (string, float) in test.below_ulp.iter() {
        let actual: T = lexical::try_parse(string).unwrap();
        assert_eq!(actual, *float);
    }

    for (string, float) in test.above_ulp.iter() {
        let actual: T = lexical::try_parse(string).unwrap();
        assert_eq!(actual, *float);
    }
}

#[test]
fn stress_tests() {
    stress_test(&STRESS_TESTS.bit24);
    stress_test(&STRESS_TESTS.bit56);
}
