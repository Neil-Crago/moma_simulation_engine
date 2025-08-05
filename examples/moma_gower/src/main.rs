// Gower toolkit


#[derive(Debug, Copy, Clone)]
struct Complex {
    re: f64, // Real part
    im: f64, // Imaginary part
}

impl Complex {
    // Multiplication: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
    fn mul(self, other: Self) -> Self {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    // Magnitude: |a+bi| = sqrt(a^2 + b^2)
    fn magnitude(self) -> f64 {
        (self.re.powi(2) + self.im.powi(2)).sqrt()
    }
}


fn calculate_u2_norm_complex(sequence: Vec<Complex>) -> f64 {
    let n = sequence.len();
    if n < 2 {
    return 0 as f64; // Norm is trivial for very short sequences
    }
    
    let mut total_sum = Complex { re: 0.0, im: 0.0 };
 
    let mut num_quads = 0;

    // Loop over all starting points x
    for x in 0..(n-1){
        // Loop over the first offset h1
        for h1 in 0..(n-1) {
            // Loop over the second offset h2
            for h2 in 0..(n-1){
                // Define the four corners of the rectangle
                let p1_idx = x;
                let p2_idx = x + h1;
                let p3_idx = x + h2;
                let p4_idx = x + h1 + h2;

                // Check if the entire rectangle is within the sequence bounds
                if p4_idx < n && p2_idx < n && p3_idx < n { 
                    // Get the values at the corner points
                    let val1 = sequence[p1_idx];
                    let val2 = sequence[p2_idx];
                    let val3 = sequence[p3_idx];
                    let val4 = sequence[p4_idx];
                    
                    

                    // Calculate the product and add to the sum
                // Use complex multiplication
                    let product = val1.mul(val2).mul(val3).mul(val4);
                    total_sum.re += product.re;
                    total_sum.im += product.im;
    
                    // Increment the counter of valid quads
                    num_quads += 1;
                }
            }
        }
    }

    // Avoid division by zero
    if num_quads == 0 {
    return 0.0;
    }

    // Calculate the average
   let average = Complex {
        re: total_sum.re / num_quads as f64,
        im: total_sum.im / num_quads as f64,
    };
    
  
     // The norm is the 4th root of the MAGNITUDE of the average.
    let norm = average.magnitude().powf(1.0 / 4.0);
    norm
}

fn path_to_complex_sequence(path: &Vec<(i32, i32)>) -> Vec<Complex> {
   let mut complex_sequence: Vec<Complex> = Vec::new();
   if path.len() < 2 { return complex_sequence; }
   
   for p in 1..path.len() {
       let dx = path[p].0 - path[p-1].0;
       let dy = path[p].1 - path[p-1].1;
       let angle = (dy as f64).atan2(dx as f64);
       
       complex_sequence.push(Complex {
           re: angle.cos(),
           im: angle.sin(),
       });
   }
   complex_sequence
}



fn main() {
    
    
    println!("\ntest with path to complex data\n");
    let straight_line = vec![(0,0), (1,0), (2,0), (3,0), (4,0), (5,0)];
    let staircase = vec![(0,0), (1,0), (1,1), (2,1), (2,2), (3,2)];
    
    let p1 = path_to_complex_sequence(&straight_line);
    let p2 = path_to_complex_sequence(&staircase);
    let c1 = calculate_u2_norm_complex(p1);
    let c2 = calculate_u2_norm_complex(p2);
   
    println!("straight_line = {c1:>3.9}");
    println!("staircase = {c2:>3.9}");
    
}
