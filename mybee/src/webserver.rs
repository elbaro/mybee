/// Serve a http page to render latency histograms in svg

// use axum::response::Html;
// use axum::routing::get;
// use plotters::prelude::{
//     BindKeyPoints, ChartBuilder, IntoDrawingArea, IntoLogRange, PathElement, SVGBackend,
// };

// tokio::spawn(async move {
//     let app = axum::Router::new().route(
//         "/",
//         get(|| async move {
//             let mut svg = String::new();
//             {
//                 // let root = SVGBackend::with_string(&mut svg, (1024, 512)).into_drawing_area();
//                 // let mut chart = ChartBuilder::on(&root)
//                 //     .margin_top(30)
//                 //     .margin_right(30)
//                 //     .x_label_area_size(40)
//                 //     .y_label_area_size(40)
//                 //     .build_cartesian_2d(
//                 //         (0f32..0.9999)
//                 //             .log_scale()
//                 //             .zero_point(1.0)
//                 //             .with_key_points(vec![0.9999, 0.999, 0.99, 0.95, 0.9, 0.5, 0.1]),
//                 //         (1_000_f32..1_000_000_000f32).log_scale(),
//                 //     )
//                 //     .unwrap();
//                 // chart
//                 //     .configure_mesh()
//                 //     .x_label_formatter(&|x| format!("{}%", *x * 100.0))
//                 //     .y_label_formatter(&|y| {
//                 //         format_duration(Duration::new(0, (*y) as u32)).to_string()
//                 //     })
//                 //     .draw()
//                 //     .unwrap();

//                 // for pair in hdrs.iter() {
//                 //     let hdr = pair.value();
//                 //     chart
//                 //         .draw_series(LineSeries::new(
//                 //             hdr.iter_quantiles(1)
//                 //                 .map(|q| {
//                 //                     (
//                 //                         (q.quantile() as f32).clamp(0.0, 0.9999),
//                 //                         (q.value_iterated_to() as f32)
//                 //                             .min(1_000_000_000_f32)
//                 //                             .max(1_000_f32),
//                 //                     )
//                 //                 })
//                 //                 .map(|x| {
//                 //                     dbg!(x);
//                 //                     x
//                 //                 }),
//                 //             &plotters::style::RED,
//                 //         ))
//                 //         .unwrap()
//                 //         .label(pair.key())
//                 //         .legend(|(x, y)| {
//                 //             PathElement::new(vec![(x, y), (x + 20, y)], &plotters::style::RED)
//                 //         });
//                 // }
//                 // chart
//                 //     .configure_series_labels()
//                 //     .border_style(&plotters::style::BLACK)
//                 //     .draw()
//                 //     .unwrap();
//             } // drop SVGBackend

//             Html(svg)
//         }),
//     );
//     axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// });
