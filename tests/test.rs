extern crate assert_cmd;
use assert_cmd::Command;

fn test_stdout(args: Vec<&str>, expected_stdout: &'static str){
    Command::cargo_bin("loop").unwrap()
    .args(args)
     .assert().success().stdout(expected_stdout);
}

#[test]
fn item(){
    test_stdout(
        vec!["--for=a,b,c", "--", "echo $ITEM"],
        "a\n\
         b\n\
         c\n");
}

#[test]
fn counter() {
    test_stdout(
        vec!["--for=a,b,c", "--", "echo $COUNT"],
        "0\n\
         1\n\
         2\n");
    test_stdout(
        vec!["--for=a,b,c", "--count-by", "2", "--", "echo $COUNT"],
        "0\n\
         2\n\
         4\n");
    test_stdout(
        vec!["--for=a,b,c", "--count-by", "2", "--offset", "10", "--", "echo $COUNT"],
        "10\n\
         12\n\
         14\n");
    test_stdout(
        vec!["--for=a,b,c", "--count-by", "1.1", "--", "echo $COUNT"],
        "0.0\n\
         1.1\n\
         2.2\n");
    test_stdout(
        vec!["--for=a,b,c", "--count-by", "2", "--", "echo $COUNT $ACTUALCOUNT"],
        "0 0\n\
         2 1\n\
         4 2\n");
}

#[test]
fn summary(){
    test_stdout(
        vec!["--for=true,false,true", "--summary", "--", "$ITEM"],
        "Total runs:\t3\n\
         Successes:\t2\n\
         Failures:\t1 (-1)\n");
    test_stdout(
        vec!["--for=true,true,true", "--summary", "--", "$ITEM"],
        "Total runs:\t3\n\
         Successes:\t3\n\
         Failures:\t0\n");
}

#[test]
fn only_last(){
    test_stdout(
        vec!["--for=a,b,c", "--only-last", "--", "echo $COUNT"],
        "2\n");
    test_stdout(
        vec!["--for=a,b,c", "--only-last", "--", "echo $COUNT"],
        "2\n");
}

#[test]
fn no_of_iterations(){
    test_stdout(
        vec!["--num", "4", "--", "echo four"],
        "four\n\
         four\n\
         four\n\
         four\n");
}

#[test]
fn until_contains(){
    test_stdout(
        vec!["--for=ferras,ferres,ferris,ferros,ferrus",
             "--until-contains", "is", "--", "echo $ITEM"],
        "ferras\n\
         ferres\n\
         ferris\n");
}

#[test]
fn until_changes(){
    test_stdout(
        vec!["--for=1,1,1,2,1,2",
             "--until-changes", "--", "echo $ITEM"],
        "1\n\
         1\n\
         1\n\
         2\n");
}

#[test]
fn until_same(){
    test_stdout(
        vec!["--for=1,2,3,4,5,5,6,7",
             "--until-same", "--only-last", "--", "echo $ITEM"],
         "5\n");
}

#[test]
fn until_success(){
    test_stdout(
        vec!["--for=false,false,false,true,false,false", "--until-success", "--summary", "--", "$ITEM"],
        "Total runs:\t4\n\
         Successes:\t1\n\
         Failures:\t3 (-1, -1, -1)\n");
}

#[test]
fn until_fail(){
    test_stdout(
        vec!["--for=true,true,false,true,true,true", "--until-fail", "--summary", "--", "$ITEM"],
        "Total runs:\t3\n\
         Successes:\t2\n\
         Failures:\t1 (-1)\n");
}

#[test]
fn until_error(){
    test_stdout(
        vec!["--for=true,true,false,true,true,true", "--until-error", "--summary", "--", "$ITEM"],
        "Total runs:\t3\n\
         Successes:\t2\n\
         Failures:\t1 (-1)\n");
}
