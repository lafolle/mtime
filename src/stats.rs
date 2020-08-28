use super::RunMetrics;

pub struct ContextStat {
    pub wall: SummaryStat,
    pub user: SummaryStat,
    pub sys: SummaryStat,
}

pub struct SummaryStat {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl SummaryStat {
    pub fn new(numbers: Vec<i64>) -> Self {
        Self {
            mean: mean(&numbers) / 1e6,
            std_dev: std_dev(&numbers) / 1e6,
            min: *min(&numbers).unwrap() as f64 / 1e6,
            median: median(&numbers) / 1e6,
            max: *max(&numbers).unwrap() as f64 / 1e6,
        }
    }
}

pub fn stats(usages: Vec<RunMetrics>) -> ContextStat {
    let wall_usages: Vec<i64> = usages
        .iter()
        .map(|ru| ru.wall_clock_dur.as_micros() as i64)
        .collect();
    let user_usages: Vec<i64> = usages.iter().map(|ru| ru.rusage.user_tv_usec).collect();
    let system_usages: Vec<i64> = usages.iter().map(|ru| ru.rusage.system_tv_usec).collect();

    ContextStat {
        wall: SummaryStat::new(wall_usages),
        user: SummaryStat::new(user_usages),
        sys: SummaryStat::new(system_usages),
    }
}

fn mean(data: &Vec<i64>) -> f64 {
    let sum: i64 = data.iter().sum();

    sum as f64 / data.len() as f64
}

fn min(data: &Vec<i64>) -> Option<&i64> {
    data.iter().min()
}

fn max(data: &Vec<i64>) -> Option<&i64> {
    data.iter().max()
}

fn median(data: &Vec<i64>) -> f64 {

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

fn std_dev(data: &Vec<i64>) -> f64 {

    let avg = mean(data);

    let x: f64= data.iter().map(|n| (*n as f64 - avg) * (*n as f64 - avg)).collect::<Vec<f64>>().iter().sum();

    (x / data.len() as f64).sqrt()

}
