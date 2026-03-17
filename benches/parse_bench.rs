use content_tag::{Options, Preprocessor};
use std::time::Instant;

fn bench_parse(name: &str, src: &str, iterations: u32) -> f64 {
    // Warmup
    for _ in 0..100 {
        let p = Preprocessor::new();
        let _ = p.parse(src, Options::default());
    }

    // Run 3 rounds, take the minimum
    let mut best = f64::MAX;
    for _ in 0..3 {
        let start = Instant::now();
        for _ in 0..iterations {
            let p = Preprocessor::new();
            let _ = p.parse(src, Options::default());
        }
        let elapsed = start.elapsed();
        let per_iter = elapsed.as_nanos() as f64 / iterations as f64;
        if per_iter < best {
            best = per_iter;
        }
    }

    println!(
        "{:<55} {:>8.1}µs per parse  ({} chars)",
        name,
        best / 1000.0,
        src.len(),
    );
    best / 1000.0
}

fn main() {
    // Global warmup: run a few hundred parses to warm CPU caches
    // before any measured benchmarks.
    {
        let w = "import Component from '@glimmer/component';\nclass C extends Component { <template>hi</template> }";
        for _ in 0..500 {
            let p = Preprocessor::new();
            let _ = p.parse(w, Options::default());
        }
    }

    // The same component is used as baseline across all tests.
    let base_component = r#"
import Component from '@glimmer/component';
class Comp extends Component {
  <template>
    <div class="container">
      <h1>{{this.title}}</h1>
      <p>{{this.description}}</p>
    </div>
  </template>
}
"#;

    // =========================================================
    // Test 1: Scaling by number of templates
    //   Same component repeated N times.
    // =========================================================
    println!("=== Scaling by template count ===\n");

    for repeats in [1, 2, 5, 10, 20] {
        let src = base_component.repeat(repeats);
        bench_parse(
            &format!("{} templates ({} chars)", repeats, src.len()),
            &src,
            3000,
        );
    }

    // =========================================================
    // Test 2: Scaling by template content size
    //   Same component, but with extra rows inside the template.
    // =========================================================
    println!("\n=== Scaling by template content size ===\n");

    let extra_row = "      <div class=\"item\">{{this.value}}</div>\n";

    // Baseline: the component as-is (0 extra rows)
    bench_parse(
        &format!("0 extra rows ({} chars)", base_component.len()),
        base_component,
        3000,
    );

    for num_rows in [10, 50, 200] {
        let extra_content = extra_row.repeat(num_rows);
        let src = base_component.replace(
            "      <p>{{this.description}}</p>",
            &format!("      <p>{{{{this.description}}}}</p>\n{}", extra_content),
        );
        bench_parse(
            &format!("{} extra rows inside template ({} chars)", num_rows, src.len()),
            &src,
            3000,
        );
    }

    // =========================================================
    // Test 3: Scaling by JS code before the template
    //   Same component, but with extra JS lines before it.
    // =========================================================
    println!("\n=== Scaling by JS code before template ===\n");

    let extra_line = "const x = 'some padding code to increase byte offset';\n";

    // Baseline: the component as-is (0 extra lines)
    bench_parse(
        &format!("0 extra lines ({} chars)", base_component.len()),
        base_component,
        3000,
    );

    for num_lines in [10, 50, 200] {
        let prefix = extra_line.repeat(num_lines);
        let src = format!("{}{}", prefix, base_component);
        bench_parse(
            &format!(
                "{} extra JS lines before template ({} chars)",
                num_lines,
                src.len()
            ),
            &src,
            3000,
        );
    }

    // =========================================================
    // Test 4: Typical real-world files
    // =========================================================
    println!("\n=== Typical files ===\n");

    let no_template = r#"
import { tracked } from '@glimmer/tracking';
import { action } from '@ember/object';
import Service, { service } from '@ember/service';

export default class AuthService extends Service {
  @service declare session: any;
  @tracked count = 0;

  @action
  increment() { this.count++; }

  get doubled() { return this.count * 2; }
}"#;

    bench_parse(
        &format!("base component (1 template, {} chars)", base_component.len()),
        base_component,
        5000,
    );
    bench_parse("utility file (no template)", no_template, 5000);
}
