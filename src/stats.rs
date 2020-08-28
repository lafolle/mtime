pub fn mean(data: &Vec<i64>) -> f64 {
    let sum: i64 = data.iter().sum();

    sum as f64 / data.len() as f64
}

pub fn min(data: &Vec<i64>) -> Option<&i64> {
    data.iter().min()
}

pub fn max(data: &Vec<i64>) -> Option<&i64> {
    data.iter().max()
}

pub fn median(data: &Vec<i64>) -> f64 {

    let mut cloned = data.clone();

    cloned.sort();

    let len = data.len();
    let middle = len / 2;
    if len % 2 == 1 {
        cloned[middle] as f64
    } else {
        cloned[middle] as f64 + cloned[middle-1] as f64 / 2 as f64
    }
}

pub fn std_dev(data: &Vec<i64>) -> f64 {

    let avg = mean(data);

    let x: f64= data.iter().map(|n| (*n as f64 - avg) * (*n as f64 - avg)).collect::<Vec<f64>>().iter().sum();

    (x / data.len() as f64).sqrt()

}
