// ASTRA.OS - Browser OS Kernel with Userspace Support
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm_const)]

extern crate alloc;
// Custom std backend implemented in rust-std-fork/library/std/src/sys/pal/astra_os/
// Can be enabled once libc dependency issues are resolved

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle, Circle},
    Drawable,
};

mod drivers;
mod gdt;
mod interrupts;
mod memory;
mod serial;
// mod ui; // TODO: UI 모듈을 픽셀 그래픽에 맞게 업데이트 필요
mod network;
mod process;
mod syscall;
mod simple_html;
mod userspace_code;
mod html;
mod keyboard;
mod fs;
mod css;
mod layout;
mod resource;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 시리얼 포트 초기화
    serial::init();
    serial_println!("ASTRA.OS v0.2.0 - Phase 4");
    serial_println!("Kernel starting...");
    serial_println!("Boot info physical_memory_offset: {:#x}", boot_info.physical_memory_offset);

    serial_println!("Initializing memory...");
    // 메모리 초기화
    memory::init(boot_info);
    serial_println!("Memory initialized");

    serial_println!("Initializing GDT...");
    // GDT 초기화 (Ring 3 세그먼트 포함)
    gdt::init();
    serial_println!("GDT initialized with userspace segments");

    serial_println!("Initializing interrupts...");
    // 인터럽트 초기화
    interrupts::init();
    serial_println!("Interrupts initialized");

    serial_println!("Initializing mouse...");
    // 마우스 드라이버 초기화
    drivers::mouse::init();
    serial_println!("Mouse initialized");

    serial_println!("Initializing network stack...");
    // 네트워크 스택 초기화
    network::init();
    serial_println!("Network stack initialized");

    serial_println!("Initializing framebuffer (VGA Mode 13h)...");
    // 프레임버퍼 초기화 (VGA Mode 13h: 320x200, 256색)
    drivers::framebuffer::init();
    serial_println!("Framebuffer initialized");

    serial_println!("Initializing filesystem...");
    // TAR 아카이브를 바이너리로 임베드
    static INITRAMFS: &[u8] = include_bytes!("../../initramfs.tar");
    match fs::init(INITRAMFS) {
        Ok(()) => serial_println!("Filesystem initialized"),
        Err(e) => serial_println!("Filesystem init error: {}", e),
    }

    // Phase 1 테스트: Font rendering, HTML rendering
    serial_println!("\n=== Phase 1 Tests ===");
    test_phase1_features();

    // Phase 2 테스트: TAR filesystem
    serial_println!("\n=== Phase 2 Tests ===");
    test_phase2_filesystem();

    // Phase 3 테스트: CSS parser and selector matching
    serial_println!("\n=== Phase 3 Tests ===");
    test_phase3_css();
    test_phase4_layout();
    test_phase5_network();

    serial_println!("\nGoing to Ring 3...\n");

    // Jump to userspace (Ring 3)
    jump_to_userspace();
}

/// Phase 1 기능 테스트
fn test_phase1_features() {
    use embedded_graphics::pixelcolor::Rgb888;

    // Test 1: Font Rendering
    serial_println!("[TEST 1] Font Rendering");
    drivers::framebuffer::clear_screen(Rgb888::new(0, 0, 0));

    drivers::framebuffer::draw_string(
        "ASTRA.OS Browser v0.2.0",
        10,
        10,
        Rgb888::new(255, 255, 255)
    );

    drivers::framebuffer::draw_string(
        "Phase 1: Font + HTML",
        10,
        25,
        Rgb888::new(100, 200, 255)
    );

    serial_println!("  Font rendering: OK");

    // Test 2: HTML Rendering
    serial_println!("[TEST 2] HTML Rendering");
    let test_html = r#"
        <html>
            <body>
                <h1>Welcome to ASTRA.OS!</h1>
                <p>A minimal browser OS written in Rust</p>
                <p>Features: HTML parsing and rendering</p>
            </body>
        </html>
    "#;

    use alloc::string::ToString;
    html::renderer::render_html_string(test_html, 320);
    serial_println!("  HTML rendering: OK");

    // Test 3: Keyboard buffer (키보드 입력은 userspace에서 테스트)
    serial_println!("[TEST 3] Keyboard Input System");
    serial_println!("  Keyboard buffer initialized: {}",
        keyboard::KEYBOARD_BUFFER.lock().available() == 0);
    serial_println!("  sys_read implementation: Ready");
    serial_println!("=== Phase 1 Tests Complete ===\n");
}

/// Phase 2 기능 테스트: TAR filesystem
fn test_phase2_filesystem() {
    use alloc::vec::Vec;

    serial_println!("[TEST 1] File listing");
    // VFS에서 TAR 파일 목록 확인 (이미 init에서 출력됨)
    serial_println!("  File listing: OK");

    serial_println!("[TEST 2] File open/read/close");
    // index.html 파일 열기
    match fs::open("index.html") {
        Ok(fd) => {
            serial_println!("  Opened index.html as FD {}", fd.0);

            // 파일 읽기 (작은 버퍼로 여러 번)
            let mut buffer = Vec::new();
            let mut temp_buf = [0u8; 128];

            loop {
                match fs::read(fd, &mut temp_buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        buffer.extend_from_slice(&temp_buf[..n]);
                        serial_println!("  Read {} bytes", n);
                    }
                    Err(e) => {
                        serial_println!("  Read error: {}", e);
                        break;
                    }
                }
            }

            serial_println!("  Total read: {} bytes", buffer.len());

            // 파일 내용 출력 (처음 100바이트만)
            if let Ok(content) = core::str::from_utf8(&buffer[..core::cmp::min(100, buffer.len())]) {
                serial_println!("  Content preview: {}", content);
            }

            // 파일 닫기
            match fs::close(fd) {
                Ok(()) => serial_println!("  Closed index.html"),
                Err(e) => serial_println!("  Close error: {}", e),
            }
        }
        Err(e) => {
            serial_println!("  Open error: {}", e);
        }
    }

    serial_println!("[TEST 3] Invalid file");
    match fs::open("nonexistent.html") {
        Ok(_) => serial_println!("  ERROR: Should not open nonexistent file"),
        Err(e) => serial_println!("  Correctly rejected: {}", e),
    }

    serial_println!("=== Phase 2 Tests Complete ===\n");
}

/// Phase 3 기능 테스트: CSS parser and selector matching
fn test_phase3_css() {
    use alloc::string::String;
    use alloc::vec::Vec;

    serial_println!("[TEST 1] CSS Color Parsing");
    // Test hex colors
    let red = css::Color::from_hex("#FF0000");
    assert!(red.is_some());
    serial_println!("  Hex color parsing (#FF0000): OK");

    let red_short = css::Color::from_hex("#F00");
    assert!(red_short.is_some());
    serial_println!("  Short hex color (#F00): OK");

    // Test named colors
    let white = css::Color::from_name("white");
    assert!(white.is_some());
    serial_println!("  Named color (white): OK");

    serial_println!("[TEST 2] CSS Parser");
    // Parse simple CSS
    let css_text = r#"
        h1 {
            color: #00FFFF;
            font-size: 20;
            margin: 10;
        }
        p {
            color: white;
            margin: 5;
        }
    "#;

    let stylesheet = css::parse_css(css_text);
    serial_println!("  Parsed {} rules", stylesheet.rules.len());

    // Check h1 rule
    assert_eq!(stylesheet.rules[0].selectors[0], "h1");
    assert_eq!(stylesheet.rules[0].declarations.len(), 3);
    serial_println!("  H1 rule: OK ({} declarations)", stylesheet.rules[0].declarations.len());

    // Check p rule
    assert_eq!(stylesheet.rules[1].selectors[0], "p");
    serial_println!("  P rule: OK");

    serial_println!("[TEST 3] Selector Matching");
    // Test tag selector
    let tag_selector = css::parse_selector("h1");
    serial_println!("  Tag selector parsed: {:?}", tag_selector);

    // Test class selector
    let class_selector = css::parse_selector(".my-class");
    serial_println!("  Class selector parsed: {:?}", class_selector);

    // Test ID selector
    let id_selector = css::parse_selector("#my-id");
    serial_println!("  ID selector parsed: {:?}", id_selector);

    // Test matching
    let mut attrs = Vec::new();
    attrs.push((String::from("class"), String::from("my-class other")));
    let element = html::ElementData {
        tag_name: String::from("div"),
        attributes: attrs,
    };

    let matches_div = css::matches_selector(&element, &css::parse_selector("div"));
    serial_println!("  Matches 'div': {}", matches_div);
    assert!(matches_div);

    let matches_class = css::matches_selector(&element, &css::parse_selector(".my-class"));
    serial_println!("  Matches '.my-class': {}", matches_class);
    assert!(matches_class);

    serial_println!("[TEST 4] Style Computation");
    // Create element
    let attrs = Vec::new();
    let h1_element = html::ElementData {
        tag_name: String::from("h1"),
        attributes: attrs,
    };

    // Compute style
    let computed = css::compute_style(&h1_element, &stylesheet);
    serial_println!("  Computed {} properties for h1", computed.properties.len());

    // Check color property
    if let Some(css::PropertyValue::Color(color)) = computed.get("color") {
        serial_println!("  H1 color: RGB({}, {}, {})", color.r, color.g, color.b);
    }

    // Check font-size property
    if let Some(css::PropertyValue::Length(size)) = computed.get("font-size") {
        serial_println!("  H1 font-size: {}px", size);
    }

    serial_println!("[TEST 5] Default Stylesheet");
    let default_styles = css::Stylesheet::default_styles();
    serial_println!("  Default stylesheet has {} rules", default_styles.rules.len());

    serial_println!("=== Phase 3 Tests Complete ===\n");
}

/// Phase 4 기능 테스트: Layout Engine (Box Model + Block/Inline Layout)
fn test_phase4_layout() {
    use alloc::string::String;
    use alloc::vec::Vec;
    use alloc::boxed::Box;

    serial_println!("=== Phase 4 Tests ===");

    serial_println!("[TEST 1] Box Model - Dimensions");
    // Test basic dimensions
    let mut dims = layout::Dimensions::new();
    dims.content = layout::Rect::new(10, 10, 100, 50);
    dims.padding = layout::EdgeSizes::uniform(5);
    dims.border = layout::EdgeSizes::uniform(2);
    dims.margin = layout::EdgeSizes::uniform(10);

    serial_println!("  Content box: {}x{} at ({}, {})",
        dims.content.width, dims.content.height,
        dims.content.x, dims.content.y);

    let border_width = dims.border_box_width();
    let margin_width = dims.margin_box_width();
    serial_println!("  Border box width: {} (expected: 114)", border_width);
    serial_println!("  Margin box width: {} (expected: 134)", margin_width);

    assert_eq!(border_width, 114); // 100 + 10(padding) + 4(border)
    assert_eq!(margin_width, 134); // 114 + 20(margin)
    serial_println!("  Box model calculations: OK");

    serial_println!("[TEST 2] Box Model - Rectangles");
    let padding_box = dims.padding_box();
    let border_box = dims.border_box();
    let margin_box = dims.margin_box();

    serial_println!("  Padding box: {}x{}", padding_box.width, padding_box.height);
    serial_println!("  Border box: {}x{}", border_box.width, border_box.height);
    serial_println!("  Margin box: {}x{}", margin_box.width, margin_box.height);

    assert_eq!(padding_box.width, 110); // 100 + 10
    assert_eq!(border_box.width, 114);  // 110 + 4
    assert_eq!(margin_box.width, 134);  // 114 + 20
    serial_println!("  Rectangle calculations: OK");

    serial_println!("[TEST 3] Layout Tree Construction");
    // Create simple HTML tree
    let body_attrs = Vec::new();
    let body_node = html::Node::element(
        String::from("body"),
        body_attrs,
        Vec::new()
    );

    // Create stylesheet
    let stylesheet = css::Stylesheet::default_styles();

    // Build layout tree
    if let Some(layout_tree) = layout::build_layout_tree(&body_node, &stylesheet) {
        serial_println!("  Layout tree created successfully");
        serial_println!("  Root box type: {:?}", layout_tree.box_type);
        serial_println!("  Root element: {:?}", layout_tree.element_name);
        assert_eq!(layout_tree.box_type, layout::BoxType::Block);
        serial_println!("  Layout tree construction: OK");
    } else {
        serial_println!("  ERROR: Failed to create layout tree");
    }

    serial_println!("[TEST 4] Block Layout Calculation");
    // Create a simple block box with style
    let mut style = css::ComputedStyle::new();
    style.set(String::from("width"), css::PropertyValue::Length(200));
    style.set(String::from("margin"), css::PropertyValue::Length(15));
    style.set(String::from("padding"), css::PropertyValue::Length(10));

    let mut block_box = Box::new(layout::LayoutBox::new(layout::BoxType::Block, style));
    block_box.element_name = Some(String::from("div"));

    // Create containing block (like browser viewport)
    let containing_block = layout::Dimensions {
        content: layout::Rect::new(0, 0, 800, 600),
        ..layout::Dimensions::default()
    };

    // Calculate layout
    layout::layout_tree(&mut block_box, containing_block);

    serial_println!("  Block width: {} (expected: 200)", block_box.dimensions.content.width);
    serial_println!("  Block position: ({}, {})",
        block_box.dimensions.content.x,
        block_box.dimensions.content.y);
    serial_println!("  Margin: {}", block_box.dimensions.margin.left);
    serial_println!("  Padding: {}", block_box.dimensions.padding.left);

    assert_eq!(block_box.dimensions.content.width, 200);
    assert_eq!(block_box.dimensions.margin.left, 15);
    assert_eq!(block_box.dimensions.padding.left, 10);
    serial_println!("  Block layout calculation: OK");

    serial_println!("[TEST 5] Complete Layout Tree");
    // Create a more complex tree
    let mut h1_attrs = Vec::new();
    h1_attrs.push((String::from("class"), String::from("title")));

    let h1_node = html::Node::element(
        String::from("h1"),
        h1_attrs,
        Vec::new()
    );

    if let Some(mut h1_layout) = layout::build_layout_tree(&h1_node, &stylesheet) {
        // Layout the h1 element
        layout::layout_tree(&mut h1_layout, containing_block);

        serial_println!("  H1 layout computed successfully");
        serial_println!("  H1 dimensions: {}x{}",
            h1_layout.dimensions.content.width,
            h1_layout.dimensions.content.height);
        serial_println!("  Complete layout tree: OK");
    }

    serial_println!("=== Phase 4 Tests Complete ===\n");
}

fn test_phase5_network() {
    use alloc::string::String;
    use alloc::vec::Vec;

    serial_println!("=== Phase 5 Tests ===");

    serial_println!("[TEST 1] URL Parser - HTTP URL");
    let url = network::Url::parse("http://example.com/path/to/file.html").unwrap();

    serial_println!("  Scheme: {}", url.scheme);
    serial_println!("  Host: {:?}", url.host);
    serial_println!("  Path: {}", url.path);

    assert_eq!(url.scheme, "http");
    assert_eq!(url.host, Some(String::from("example.com")));
    assert_eq!(url.path, "/path/to/file.html");
    assert_eq!(url.port, None);
    assert_eq!(url.port_or_default(), Some(80));
    serial_println!("  HTTP URL parsing: OK");

    serial_println!("[TEST 2] URL Parser - HTTPS URL with port");
    let url = network::Url::parse("https://example.com:8443/api/data?key=value&foo=bar#section").unwrap();

    serial_println!("  Scheme: {}", url.scheme);
    serial_println!("  Host: {:?}", url.host);
    serial_println!("  Port: {:?}", url.port);
    serial_println!("  Path: {}", url.path);
    serial_println!("  Query: {:?}", url.query);
    serial_println!("  Fragment: {:?}", url.fragment);

    assert_eq!(url.scheme, "https");
    assert_eq!(url.host, Some(String::from("example.com")));
    assert_eq!(url.port, Some(8443));
    assert_eq!(url.path, "/api/data");
    assert_eq!(url.query, Some(String::from("key=value&foo=bar")));
    assert_eq!(url.fragment, Some(String::from("section")));
    serial_println!("  HTTPS URL with port, query, fragment: OK");

    serial_println!("[TEST 3] URL Parser - File URL");
    let url = network::Url::parse("file:///home/user/index.html").unwrap();

    serial_println!("  Scheme: {}", url.scheme);
    serial_println!("  Path: {}", url.path);

    assert_eq!(url.scheme, "file");
    assert_eq!(url.host, None);
    assert_eq!(url.path, "/home/user/index.html");
    serial_println!("  File URL parsing: OK");

    serial_println!("[TEST 4] URL to String");
    let url = network::Url::parse("http://example.com:8080/path?query=1#frag").unwrap();
    let url_string = url.to_string();

    serial_println!("  URL string: {}", url_string);
    assert!(url_string.contains("http://"));
    assert!(url_string.contains("example.com"));
    assert!(url_string.contains(":8080"));
    assert!(url_string.contains("/path"));
    assert!(url_string.contains("?query=1"));
    assert!(url_string.contains("#frag"));
    serial_println!("  URL to string: OK");

    serial_println!("[TEST 5] HTTP Request - GET method");
    let url = network::Url::parse("http://example.com/index.html").unwrap();
    let request = network::HttpRequest::get(url.clone());

    serial_println!("  Method: {:?}", request.method);
    serial_println!("  URL: {}", request.url.to_string());
    serial_println!("  Headers count: {}", request.headers.len());

    assert_eq!(request.method, network::HttpMethod::GET);
    assert_eq!(request.url, url);
    assert!(request.headers.len() >= 3); // Host, User-Agent, Accept
    serial_println!("  HTTP GET request: OK");

    serial_println!("[TEST 6] HTTP Request String");
    let url = network::Url::parse("http://example.com/api/data?key=value").unwrap();
    let request = network::HttpRequest::get(url);
    let request_string = request.to_request_string();

    serial_println!("  Request string (first 50 chars): {}...",
        if request_string.len() > 50 { &request_string[..50] } else { &request_string });

    assert!(request_string.starts_with("GET /api/data?key=value HTTP/1.1\r\n"));
    assert!(request_string.contains("Host: example.com\r\n"));
    assert!(request_string.contains("User-Agent: ASTRA.OS-Browser/0.1\r\n"));
    assert!(request_string.ends_with("\r\n\r\n"));
    serial_println!("  HTTP request string: OK");

    serial_println!("[TEST 7] HTTP Response Parsing");
    let response_data = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\n\r\n<html></html>";
    let response = network::parse_response(response_data).unwrap();

    serial_println!("  Status code: {}", response.status.code);
    serial_println!("  Status reason: {}", response.status.reason);
    serial_println!("  Headers count: {}", response.headers.len());
    serial_println!("  Body length: {}", response.body.len());

    assert_eq!(response.status.code, 200);
    assert_eq!(response.status.reason, "OK");
    assert!(response.status.is_success());
    assert_eq!(response.get_header("Content-Type"), Some(&String::from("text/html")));
    assert_eq!(response.content_length(), Some(13));
    assert_eq!(response.body, b"<html></html>");
    serial_println!("  HTTP response parsing: OK");

    serial_println!("[TEST 8] HTTP Status - Redirects and Errors");
    let redirect_data = b"HTTP/1.1 301 Moved Permanently\r\nLocation: /new-path\r\n\r\n";
    let redirect_response = network::parse_response(redirect_data).unwrap();

    serial_println!("  Redirect status: {}", redirect_response.status.code);
    assert_eq!(redirect_response.status.code, 301);
    assert!(redirect_response.status.is_redirect());
    serial_println!("  HTTP redirect detection: OK");

    let error_data = b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nNot found";
    let error_response = network::parse_response(error_data).unwrap();

    serial_println!("  Error status: {}", error_response.status.code);
    assert_eq!(error_response.status.code, 404);
    assert!(error_response.status.is_client_error());
    serial_println!("  HTTP error detection: OK");

    serial_println!("[TEST 9] Resource Type Detection - From URL");
    let html_url = network::Url::parse("http://example.com/page.html").unwrap();
    let css_url = network::Url::parse("http://example.com/style.css").unwrap();
    let js_url = network::Url::parse("http://example.com/script.js").unwrap();
    let img_url = network::Url::parse("http://example.com/image.png").unwrap();

    assert_eq!(resource::ResourceType::from_url(&html_url), resource::ResourceType::Html);
    assert_eq!(resource::ResourceType::from_url(&css_url), resource::ResourceType::Css);
    assert_eq!(resource::ResourceType::from_url(&js_url), resource::ResourceType::JavaScript);
    assert_eq!(resource::ResourceType::from_url(&img_url), resource::ResourceType::Image);
    serial_println!("  Resource type from URL: OK");

    serial_println!("[TEST 10] Resource Type Detection - From Content-Type");
    assert_eq!(
        resource::ResourceType::from_content_type("text/html; charset=utf-8"),
        resource::ResourceType::Html
    );
    assert_eq!(
        resource::ResourceType::from_content_type("text/css"),
        resource::ResourceType::Css
    );
    assert_eq!(
        resource::ResourceType::from_content_type("application/javascript"),
        resource::ResourceType::JavaScript
    );
    assert_eq!(
        resource::ResourceType::from_content_type("image/png"),
        resource::ResourceType::Image
    );
    serial_println!("  Resource type from content-type: OK");

    serial_println!("[TEST 11] Resource Loader - Initialization");
    let loader = resource::ResourceLoader::new();
    serial_println!("  Resource loader created successfully");
    serial_println!("  Resource loader: OK");

    serial_println!("[TEST 12] Resource - Creation and String Conversion");
    let url = network::Url::parse("http://example.com/test.html").unwrap();
    let data = b"<html><body>Test</body></html>".to_vec();
    let resource = resource::Resource::new(url.clone(), data);

    serial_println!("  Resource URL: {}", resource.url.to_string());
    serial_println!("  Resource type: {:?}", resource.resource_type);

    assert_eq!(resource.resource_type, resource::ResourceType::Html);
    assert_eq!(resource.url, url);

    if let Some(content) = resource.as_string() {
        serial_println!("  Resource content: {}", content);
        assert_eq!(content, "<html><body>Test</body></html>");
    }
    serial_println!("  Resource creation: OK");

    serial_println!("=== Phase 5 Tests Complete ===\n");
}

/// Jump from Ring 0 (kernel) to Ring 3 (userspace)
fn jump_to_userspace() -> ! {
    use x86_64::VirtAddr;
    use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};

    // Get userspace entry point
    let userspace_entry = userspace_code::get_userspace_entry();
    serial_println!("Userspace entry point: {:#x}", userspace_entry);

    // Allocate user stack with page alignment to avoid overlapping with GDT
    #[repr(align(4096))]
    struct UserStack([u8; 16384]);  // 16KB stack, page-aligned

    static mut USER_STACK: UserStack = UserStack([0; 16384]);
    let user_stack_top = unsafe {
        VirtAddr::from_ptr(USER_STACK.0.as_ptr()).as_u64() + USER_STACK.0.len() as u64
    };
    serial_println!("User stack: {:#x}", user_stack_top);

    // Mark userspace code and stack pages as USER accessible
    serial_println!("Marking userspace pages as USER accessible...");
    unsafe {
        // Mark code pages as USER accessible
        // Mark multiple pages to ensure all userspace code/data is accessible
        let code_start = VirtAddr::new(userspace_entry);
        let code_page = Page::containing_address(code_start);
        serial_println!("    Code at {:#x}, starting from page {:#x}", userspace_entry, code_page.start_address().as_u64());

        // Mark the code page and adjacent pages to cover all userspace code/data
        // Need to mark all pages from code start to stack start (0x215000 - 0x224000 = 15 pages)
        for i in 0..16 {  // Mark 16 pages (64KB) for userspace code and data
            let page: Page<Size4KiB> = code_page + i;
            memory::mark_code_page_user_accessible(page);
        }

        // Mark stack pages as USER accessible
        let stack_start = VirtAddr::from_ptr(USER_STACK.0.as_ptr());
        let stack_end = stack_start + USER_STACK.0.len() as u64;
        serial_println!("    Stack from {:#x} to {:#x}", stack_start.as_u64(), stack_end.as_u64());
        let stack_start_page = Page::containing_address(stack_start);
        let stack_end_page = Page::containing_address(stack_end - 1u64);
        serial_println!("    Stack pages from {:#x} to {:#x}", stack_start_page.start_address().as_u64(), stack_end_page.start_address().as_u64());

        for page in Page::range_inclusive(stack_start_page, stack_end_page) {
            memory::mark_data_page_user_accessible(page);
        }
    }
    serial_println!("Userspace pages marked as USER accessible");

    // Ensure kernel stack pages are properly mapped and writable
    serial_println!("Mapping kernel stack pages...");
    unsafe {
        // Get TSS info
        let tss_info = gdt::get_tss_info();
        let kernel_stack_top = tss_info.0;
        let kernel_stack_start = kernel_stack_top - 16384;  // 16KB stack
        let double_fault_stack_top = tss_info.1;
        let double_fault_stack_start = double_fault_stack_top - 16384;
        let timer_int_stack_top = tss_info.2;
        let timer_int_stack_start = timer_int_stack_top - 16384;
        let syscall_stack_top = tss_info.3;
        let syscall_stack_start = syscall_stack_top - 16384;

        serial_println!("  Kernel stack: {:#x} - {:#x}", kernel_stack_start, kernel_stack_top);
        serial_println!("  Double fault stack: {:#x} - {:#x}", double_fault_stack_start, double_fault_stack_top);
        serial_println!("  Timer interrupt stack: {:#x} - {:#x}", timer_int_stack_start, timer_int_stack_top);
        serial_println!("  Syscall stack: {:#x} - {:#x}", syscall_stack_start, syscall_stack_top);

        // Map kernel stack pages as WRITABLE (not user accessible)
        let kernel_stack_start_page = Page::containing_address(VirtAddr::new(kernel_stack_start));
        let kernel_stack_end_page = Page::containing_address(VirtAddr::new(kernel_stack_top - 1));

        for page in Page::range_inclusive(kernel_stack_start_page, kernel_stack_end_page) {
            serial_println!("  Ensuring kernel stack page {:#x} is writable", page.start_address().as_u64());
            // Kernel stack pages should already be mapped, just ensure WRITABLE flag
            memory::ensure_page_writable(page);
        }

        // Map double fault stack pages as WRITABLE
        let df_stack_start_page = Page::containing_address(VirtAddr::new(double_fault_stack_start));
        let df_stack_end_page = Page::containing_address(VirtAddr::new(double_fault_stack_top - 1));

        for page in Page::range_inclusive(df_stack_start_page, df_stack_end_page) {
            serial_println!("  Ensuring double fault stack page {:#x} is writable", page.start_address().as_u64());
            memory::ensure_page_writable(page);
        }

        // Map timer interrupt stack pages as WRITABLE
        let timer_int_stack_start_page = Page::containing_address(VirtAddr::new(timer_int_stack_start));
        let timer_int_stack_end_page = Page::containing_address(VirtAddr::new(timer_int_stack_top - 1));

        for page in Page::range_inclusive(timer_int_stack_start_page, timer_int_stack_end_page) {
            serial_println!("  Ensuring timer interrupt stack page {:#x} is writable", page.start_address().as_u64());
            memory::ensure_page_writable(page);
        }

        // Map syscall stack pages as WRITABLE
        let syscall_stack_start_page = Page::containing_address(VirtAddr::new(syscall_stack_start));
        let syscall_stack_end_page = Page::containing_address(VirtAddr::new(syscall_stack_top - 1));

        for page in Page::range_inclusive(syscall_stack_start_page, syscall_stack_end_page) {
            serial_println!("  Ensuring syscall stack page {:#x} is writable", page.start_address().as_u64());
            memory::ensure_page_writable(page);
        }

        serial_println!("  Kernel stacks mapped successfully");

        // Now test kernel stack is writable
        let test_addr = (kernel_stack_top - 8) as *mut u64;
        *test_addr = 0xDEADBEEF;
        serial_println!("  Kernel stack write test OK");
    }

    // Get user segment selectors
    let user_cs = gdt::user_code_selector().0 as u64;
    let user_ss = gdt::user_data_selector().0 as u64;

    serial_println!("User CS: {:#x}, User SS: {:#x}", user_cs, user_ss);
    serial_println!("User RIP: {:#x}, User RSP: {:#x}", userspace_entry, user_stack_top);

    // Ensure stack is 16-byte aligned
    let user_stack_top = user_stack_top & !0xF;
    serial_println!("User stack aligned to: {:#x}", user_stack_top);

    serial_println!("Executing iretq to Ring 3 with interrupts DISABLED...");
    serial_println!("Userspace will enable interrupts with STI instruction after stable entry");

    unsafe {
        core::arch::asm!(
            // Push values for iretq (in reverse order)
            "push {ss}",          // SS (user data segment)
            "push {rsp}",         // RSP (user stack pointer)
            "pushfq",             // RFLAGS (with current flags)
            "and qword ptr [rsp], ~0x200",  // Clear IF (keep interrupts disabled)
            "push {cs}",          // CS (user code segment)
            "push {rip}",         // RIP (user code entry point)
            "iretq",              // Return to Ring 3

            ss = in(reg) user_ss,
            rsp = in(reg) user_stack_top,
            cs = in(reg) user_cs,
            rip = in(reg) userspace_entry,
            options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
