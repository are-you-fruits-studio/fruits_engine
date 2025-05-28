use proc_macro::TokenStream;

#[proc_macro]
pub fn swizzling(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

const Parameters: &[&'static str] = &["x", "y", "z", "w"];
const ZeroParameterName: &'static str = "n";

//
    public static class VectorsSwizzlingProvider
    {
        public static List<Source> GenerateSwizzlings()
        {
            var results = new List<Source>();

            int[] counts = { 2, 3, 4 };

            foreach (var input in counts)
            {
                foreach (var output in counts)
                {
                    results.Add(new Source(
                        name: $"SwizzlingVector{input}To{output}Extensions.g.cs",
                        content: GenerateVectorSwizzling(input, output)));
                }
            }

            return results;
        }

        private static string GenerateVectorSwizzling(int inputCount, int outputCount)
        {
            var result = new StringBuilder();

            result.AppendLine("using UnityEngine;");
            result.AppendLine("");
            result.AppendLine("namespace AreYouFruits.VectorsSwizzling");
            result.AppendLine("{");
            result.AppendLine($"    public static class SwizzlingVector{inputCount}To{outputCount}Extensions");
            result.AppendLine("    {");
            
            foreach (var s in GenerateVectorSwizzling(outputCount, Parameters.AsSpan().Slice(0, inputCount)))
            {
                result.AppendLine("        " + s);
            }

            result.AppendLine("    }");
            result.AppendLine("}");
            
            return result.ToString();
        }


    }
    
fn GenerateVectorSwizzling(outputParametersCount: usize, parameters: &[&str]) -> Vec<String> {
    let swizzlingIndices = GenerateSwizzlingsWithZeros(outputParametersCount, parameters.len());

    let lower = parameters.iter().map(|p| p.to_lowercase()).collect::<Vec<_>>();
    let upper = parameters.iter().map(|p| p.to_uppercase()).collect::<Vec<_>>();

    var results = new List<string>();

    var zeroName = ZeroParameterName.ToUpper();

    foreach (var indices in swizzlingIndices)
    {
        if (indices.All(i => !i.HasValue))
        {
            continue;
        }
        
        var name = string.Join(string.Empty, indices.Select(i => i is { } j ? upper[j] : zeroName));
    
        var constructorArguments = string.Join(", ", indices.Select((i, position) => i is { } j ? $"v.{lower[j]}" : Parameters[position]));

        var additionalParameters = ToAdditionalParameters(indices);
        
        results.Add($"public static Vector{outputParametersCount} {name}(this Vector{parameters.Length} v{additionalParameters}) => new({constructorArguments});");
    }

    results
}

fn GenerateSwizzlingsWithZeros(outputParametersCount: usize, parametersCount: usize) -> HashSet<Vec<Option<usize>>> {
    let mut result = HashSet::new();

    for swizzlingsIndices in GenerateSwizzlingsIndices(outputParametersCount, parametersCount + 1)
    {
        result.insert(swizzlingsIndices.map(|i| i == parametersCount ? None : Some(i)).collect::<Vec<_>>());
    }

    result
}

fn GenerateSwizzlingsIndices(outputParametersCount: usize, parametersCount: usize) -> HashSet<Vec<usize>> {
    let mut swizzlings = HashSet::new();

    for (var i = 0; i < parametersCount; i++)
    {
        swizzlings.insert(new[] { i });
    }

    for i in 1..outputParametersCount {
        for indices in swizzlings.iter().collect::<Vec<_>>() {
            swizzlings.remove(indices);

            for (var j = 0; j < parametersCount; j++)
            {
                swizzlings.insert(std::iter::once(j).chain(indices).collect::<Vec<_>>());
            }
        }
    }

    swizzlings
}

fn ToAdditionalParameters(indices: Vec<Option<int>>) -> String {
    let mut result = String::new();

    for position in 0..indices.len() {
        if indices[position].is_none() {
            result.push_str(", float ");
            result.push_str(Parameters[position]);
            result.push_str(" = 0.0f");
        }
    }
    
    result
}