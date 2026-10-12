use gielinor_protocol::isaac::Isaac;

#[test]
fn matches_rsprot_across_multiple_refills() {
    let seeds: Vec<Vec<u32>> = vec![
        vec![],
        vec![0; 4],
        vec![1, 2, 3, 4],
        vec![0x80000000, 0xffffffff, 0x7fffffff, 2391],
        (0..256u32).map(|i| i.wrapping_mul(0x1234567)).collect(),
    ];
    let expected: Vec<i32> = include_str!("fixtures/rsprot-isaac.expected")
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();
    assert_eq!(expected.len(), 5125);
    for (seed_index, seed) in seeds.iter().enumerate() {
        let mut cipher = Isaac::new(seed).unwrap();
        for i in 0..1025 {
            assert_eq!(
                cipher.next_u32() as i32,
                expected[seed_index * 1025 + i],
                "seed {seed_index}, output {i}"
            );
        }
    }
}
