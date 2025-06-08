// Example usage of RegionSalaryInfo::calculate_expected_salary
use vctcareer_backend::offers::RegionSalaryInfo;

fn main() {
    let info = RegionSalaryInfo {
        min: 50_000,
        max: 600_000,
        plus_minus: 25_000,
    };

    // Example player overalls
    let overalls = [1.0, 25.0, 50.0, 75.0, 100.0];
    for overall in overalls.iter() {
        let salary = RegionSalaryInfo::calculate_expected_salary(
            *overall,        // overall_rating
            0.0,             // overall_min
            100.0,           // overall_max
            info.min as f64, // region_min_salary
            info.max as f64, // region_max_salary
        );
        println!("Overall: {:>5} => Salary: ${:>8.2}", overall, salary);
    }
}
