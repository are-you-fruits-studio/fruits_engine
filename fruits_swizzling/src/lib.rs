use proc_macro::TokenStream;

// todo: Separate <T: Copy> and <T> swizzling.

#[proc_macro]
pub fn swizzling(_item: TokenStream) -> TokenStream {
    let mut result = String::new();

    for swizzle in generate_swizzlings() {
        result.push_str(&swizzle);
        result.push('\n');
    }

    result.parse().unwrap()
}

const PARAMETERS: &[&'static str] = &["x", "y", "z", "w"];
const ZERO_PARAMETER_NAME: &'static str = "n";

fn generate_swizzlings() -> Vec<String> {
    let mut results = Vec::new();

    let counts = [2, 3, 4];

    for input in counts {
        for output in counts {
            results.push(generate_vector_swizzling_impl(input, output));
        }
    }

    results
}
        
fn generate_vector_swizzling_impl(input_count: usize, output_count: usize) -> String
{
    let mut result = String::new();
    result.push_str(&format!("impl<T: Copy> Vec{input_count}<T> {{\n"));
    
    for s in generate_vector_swizzling(output_count, &PARAMETERS[0..input_count]) {
        result.push_str(&format!("    {}\n", &s));
    }
    result.push_str("}\n");
    
    result
}

fn generate_vector_swizzling(output_parameters_count: usize, parameters: &[&str]) -> Vec<String> {
    let swizzling_indices = generate_swizzlings_with_zeros(output_parameters_count, parameters.len());

    let lower = parameters.iter().map(|p| p.to_lowercase()).collect::<Vec<_>>();

    let mut results = Vec::<String>::new();

    let zero_name = ZERO_PARAMETER_NAME;

    for indices in swizzling_indices {
        if indices.iter().all(|i| i.is_none()) {
            continue;
        }
        
        let name = indices.iter().map(|i| if let Some(j) = i { lower[*j].clone() } else { String::from(zero_name) }).collect::<Vec<_>>().join("");
    
        let constructor_arguments = indices.iter().enumerate().map(|(position, i)| if let Some(j) = i { format!("self.{}", lower[*j]) } else { String::from(PARAMETERS[position]) }).collect::<Vec<_>>().join(", ");

        let additional_parameters = to_additional_parameters(indices);
        
        results.push(format!("pub fn {name}(&self{additional_parameters}) -> Vec{output_parameters_count}<T> {{ Vec{output_parameters_count}::new({constructor_arguments}) }}"));
    }

    results
}

fn generate_swizzlings_with_zeros(output_parameters_count: usize, parameters_count: usize) -> Vec<Vec<Option<usize>>> {
    let mut result = Vec::new();

    for swizzlings_indices in generate_swizzlings_indices(output_parameters_count, parameters_count + 1) {
        result.push(swizzlings_indices.iter().map(|i| if *i == parameters_count { None } else { Some(*i) }).collect::<Vec<_>>());
    }

    result
}

fn generate_swizzlings_indices(output_parameters_count: usize, parameters_count: usize) -> Vec<Vec<usize>> {
    let mut swizzlings = Vec::new();

    for i in 0..parameters_count {
        swizzlings.push(vec![i]);
    }

    for _ in 1..output_parameters_count {
        let last_swizzlings = swizzlings.clone();
        swizzlings.clear();

        for indices in last_swizzlings {
            for j in 0..parameters_count {
                swizzlings.push(indices.iter().copied().chain(std::iter::once(j)).collect::<Vec<_>>());
            }
        }
    }

    swizzlings
}

// fn GenerateSwizzlingsIndices(outputParametersCount: usize, parametersCount: usize) -> HashSet<Vec<usize>> {
//     let mut swizzlings = HashSet::new();

//     for i in 0..parametersCount {
//         swizzlings.insert(vec![i]);
//     }

//     for i in 1..outputParametersCount {
//         let s = swizzlings.clone();
//         for indices in s {
//             swizzlings.remove(indices);

//             for j in 0..parametersCount {
//                 swizzlings.insert(std::iter::once(j).chain(indices.iter().copied()).collect::<Vec<_>>());
//             }
//         }
//     }

//     swizzlings
// }

fn to_additional_parameters(indices: Vec<Option<usize>>) -> String {
    let mut result = String::new();

    for position in 0..indices.len() {
        if indices[position].is_none() {
            result.push_str(", ");
            result.push_str(PARAMETERS[position]);
            result.push_str(": T");
        }
    }
    
    result
}