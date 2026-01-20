#[macro_export]
macro_rules! return_if {
    ($cond: expr) => {
        if $cond {
            return;
        }
    };
}

#[macro_export]
macro_rules! continue_if {
    ($cond: expr) => {
        if $cond {
            continue;
        }
    };
}

#[macro_export]
macro_rules! break_if {
    ($cond: expr) => {
        if $cond {
            break;
        }
    };
}

#[macro_export]
macro_rules! return_if_not {
    ($p: pat = $e: expr) => {
        let $p = $e else {
            return;
        };
    };
}

#[macro_export]
macro_rules! continue_if_not {
    ($p: pat = $e: expr) => {
        let $p = $e else {
            continue;
        };
    };
}

#[macro_export]
macro_rules! break_if_not {
    ($p: pat = $e: expr) => {
        let $p = $e else {
            break;
        };
    };
}
