#[allow(unused)]
use super::request_reader::*;

#[test]
fn test_transfer_smaller() {
  let mut src = vec![1u8; 5];
  let mut dest = vec![0u8; 10];

  let transferred = transfer(&mut src, &mut dest, 0);

  // [1,1,1,1,1,0,0,0,0,0]
  assert!(transferred == 5);
  assert!(src.len() == 0);
  assert!(dest[0] == 1);
  assert!(dest[1] == 1);
  assert!(dest[2] == 1);
  assert!(dest[3] == 1);
  assert!(dest[4] == 1);
  assert!(dest[5] == 0);
  assert!(dest[6] == 0);
  assert!(dest[7] == 0);
  assert!(dest[8] == 0);
  assert!(dest[9] == 0);
}

#[test]
fn test_transfer_same() {
  let mut src = vec![1u8; 5];
  let mut dest = vec![0u8; 5];

  let transferred = transfer(&mut src, &mut dest, 0);

  // [1,1,1,1,1]
  assert!(transferred == 5);
  assert!(src.len() == 0);
  assert!(dest[0] == 1);
  assert!(dest[1] == 1);
  assert!(dest[2] == 1);
  assert!(dest[3] == 1);
  assert!(dest[4] == 1);
}

#[test]
fn test_transfer_larger() {
  let mut src = vec![1u8; 10];
  let mut dest = vec![0u8; 5];

  let transferred = transfer(&mut src, &mut dest, 0);

  // [1,1,1,1,1]
  assert!(transferred == 5);
  assert!(src.len() == 5);
  assert!(dest[0] == 1);
  assert!(dest[1] == 1);
  assert!(dest[2] == 1);
  assert!(dest[3] == 1);
  assert!(dest[4] == 1);
}

#[test]
fn test_transfer_from_middle() {
  let mut src = vec![1u8; 3];
  let mut dest = vec![0u8; 10];

  let transferred = transfer(&mut src, &mut dest, 7);

  // [0,0,0,0,0,1,1,1,1,1]
  assert!(transferred == 3);
  assert!(src.len() == 0);
  assert!(dest[0] == 0);
  assert!(dest[1] == 0);
  assert!(dest[2] == 0);
  assert!(dest[3] == 0);
  assert!(dest[4] == 0);
  assert!(dest[5] == 0);
  assert!(dest[6] == 0);
  assert!(dest[7] == 1);
  assert!(dest[8] == 1);
  assert!(dest[9] == 1);
}
