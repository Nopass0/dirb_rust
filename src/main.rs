use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write, stdin};
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use chrono::prelude::*;
use tokio::sync::{mpsc, Semaphore};
use tokio::task;
use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
use tokio::net::TcpStream;
use crossterm::{execute, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, cursor, style::Stylize};
use std::io::stdout;

const MAX_CONCURRENT_REQUESTS: usize = 10000;
const COMMON_PORTS: [u16; 10] = [21, 22, 23, 25, 80, 110, 143, 443, 3306, 8080];

/// Объединяет все текстовые файлы из указанной директории в один файл, удаляя дубликаты.
fn merge_wordlists(directory: &str, output_file: &str) -> io::Result<()> {
    let mut unique_lines = HashSet::new();
    let mut total_files = 0;

    // Проверка наличия директории wordlists
    if !Path::new(directory).exists() {
        println!("Директория {} не найдена, создаем новую...", directory);
        fs::create_dir(directory)?;
    }

    // Собираем все .txt файлы в папке
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            total_files += 1;
        }
    }

    if total_files == 0 {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Нет файлов .txt в директории wordlists"));
    }

    // Создаем прогресс-бар
    let progress_bar = ProgressBar::new(total_files as u64);
    let progress_style = ProgressStyle::default_bar()
        .template("{bar:40.cyan/blue} {pos}/{len} ({eta})")
        .progress_chars("#>-");
    progress_bar.set_style(progress_style);

    // Читаем строки из всех файлов и добавляем их в HashSet для удаления дубликатов
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                unique_lines.insert(line);
            }

            progress_bar.inc(1);
        }
    }

    progress_bar.finish();

    // Записываем уникальные строки в выходной файл
    let mut output = File::create(output_file)?;
    for line in unique_lines {
        writeln!(output, "{}", line)?;
    }

    Ok(())
}

/// Асинхронная проверка на открытые порты.
async fn check_port(address: &str, port: u16) -> bool {
    let addr = format!("{}:{}", address, port);
    match addr.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                return TcpStream::connect(addr).await.is_ok();
            }
        }
        Err(_) => return false,
    }
    false
}

/// Описание портов
fn get_port_description(port: u16) -> &'static str {
    match port {
        21 => "FTP",
        22 => "SSH",
        23 => "Telnet",
        25 => "SMTP",
        80 => "HTTP",
        110 => "POP3",
        143 => "IMAP",
        443 => "HTTPS",
        3306 => "MySQL",
        8080 => "HTTP Proxy",
        _ => "Unknown",
    }
}

/// Проверка запросов к API
async fn check_api_requests(base_url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .build()?;

    let response = client.get(base_url).send().await?;
    let body = response.text().await?;

    let mut api_requests = Vec::new();

    for line in body.lines() {
        if line.contains("fetch") || line.contains("axios") || line.contains("XMLHttpRequest") {
            api_requests.push(line.to_string());
        }
    }

    Ok(api_requests)
}

fn print_status(progress_bar: &ProgressBar, processed: usize, total: usize, valid_count: usize, valid_urls: &[String], start_time: Instant) {
    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs();
    let elapsed_formatted = format!("{:02}:{:02}:{:02}", elapsed_secs / 3600, (elapsed_secs % 3600) / 60, elapsed_secs % 60);
    let remaining_urls = if valid_urls.len() > 3 {
        format!("И ещё ({}) строк", valid_urls.len() - 3)
    } else {
        String::new()
    };

    let mut display_urls = valid_urls.iter().rev().take(3).cloned().collect::<Vec<_>>();
    display_urls.reverse();

    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    println!("+------------------------------------------------------------+");
    println!("| Обработано: {}/{} | Валидные: {} | Время работы: {} |", processed, total, valid_count, elapsed_formatted);
    println!("+------------------------------------------------------------+");
    for url in display_urls {
        println!("| {:<60} |", url);
    }
    if !remaining_urls.is_empty() {
        println!("| {:<60} |", remaining_urls);
    }
    println!("+------------------------------------------------------------+");
    println!("{}", "+------------------------------------------------------------+".blue());
    println!("{}", "| Разработчик Богдан t.me/pbgal                             |".blue());
    println!("{}", "+------------------------------------------------------------+".blue());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    execute!(stdout(), EnterAlternateScreen).unwrap();

    // Объединение всех wordlists в один файл
    if let Err(e) = merge_wordlists("wordlists", "wordlist.txt") {
        eprintln!("Ошибка при объединении wordlists: {}", e);
        return Err(e.into());
    }

    // Запрос URL сайта у пользователя
    let mut input = String::new();
    println!("Введите URL сайта (например, http://example.com): ");
    stdin().read_line(&mut input)?;
    let base_url = input.trim().to_string();

    // Получение базового домена
    let base_domain = base_url.replace("http://", "").replace("https://", "").replace("/", "");

    // Запуск таймера
    let start_time = Instant::now();
    let client = Arc::new(Client::builder().build()?);
    let (sender, mut receiver) = mpsc::channel(10000);
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let results = Arc::new(Mutex::new(Vec::new()));

    // Чтение wordlist.txt и создание асинхронного потока строк
    let file = tokio::fs::File::open("wordlist.txt").await?;
    let reader = TokioBufReader::new(file);
    let lines = reader.lines();
    let mut lines_stream = lines;

    // Настройка прогресс-бара
    let progress_bar = ProgressBar::new_spinner();
    let progress_style = ProgressStyle::default_bar()
        .template("{bar:40.cyan/blue} {pos}/{len} ({eta})")
        .progress_chars("#>-");
    progress_bar.set_style(progress_style);

    let mut url_handles = Vec::new();
    let mut total_lines = 0;
    let mut valid_count = 0;
    let mut urls_to_check = Vec::new();

    // Считывание базовых URL из файла
    while let Some(line) = lines_stream.next_line().await? {
        total_lines += 1;
        let url = format!("{}/{}", base_url, line);
        urls_to_check.push(url);
    }

    // Проверка доступности URL
    for url in urls_to_check {
        let client = Arc::clone(&client);
        let sender = sender.clone();
        let semaphore = Arc::clone(&semaphore);
        let results = Arc::clone(&results);
        let progress_bar = progress_bar.clone();

        let handle = task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let response = client.get(&url).send().await;

            if let Ok(response) = response {
                if response.status().is_success() {
                    sender.send(format!("Found: {}", url)).await.unwrap();
                    let mut res = results.lock().unwrap();
                    res.push(url.clone());
                }
            }
            progress_bar.inc(1);
        });
        url_handles.push(handle);
    }

    drop(sender);

    let receiver_handle = task::spawn(async move {
        while let Some(_msg) = receiver.recv().await {
            // Мы больше не выводим сообщения в консоль здесь
        }
    });

    let results_clone = Arc::clone(&results);
    let progress_bar_clone = progress_bar.clone();

    let status_handle = task::spawn({
        let progress_bar_clone = progress_bar_clone.clone();
        async move {
            loop {
                {
                    let results = results_clone.lock().unwrap();
                    let valid_count = results.len();
                    print_status(&progress_bar_clone, progress_bar_clone.position() as usize, total_lines, valid_count, &results, start_time);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    });

    futures::future::join_all(url_handles).await;
    receiver_handle.await.unwrap();
    status_handle.abort();
    progress_bar.finish();

    // Проверка открытых портов
    let open_ports = Arc::new(Mutex::new(Vec::new()));
    let mut port_handles = Vec::new();

    for &port in COMMON_PORTS.iter() {
        let base_url = base_url.clone();
        let open_ports = Arc::clone(&open_ports);

        let handle = task::spawn(async move {
            if check_port(&base_url, port).await {
                let mut ports = open_ports.lock().unwrap();
                ports.push(port);
            }
        });
        port_handles.push(handle);
    }

    futures::future::join_all(port_handles).await;

    // Проверка запросов к API
    let api_requests = check_api_requests(&base_url).await?;

    let end_time = Instant::now();
    let duration = end_time.duration_since(start_time);

    let results = results.lock().unwrap();

    // Форматирование имени файла вывода
    let current_date = Utc::now().format("%Y-%m-%d").to_string();
    let output_filename = format!("output_{}_{}.txt", base_domain, current_date);
    let mut output_file = File::create(&output_filename)?;

    // Запись результатов в файл
    for url in results.iter() {
        writeln!(output_file, "{}", url)?;
    }

    writeln!(output_file, "-----------------")?;

    let open_ports = open_ports.lock().unwrap();
    if !open_ports.is_empty() {
        writeln!(output_file, "Открытые порты:")?;
        for &port in open_ports.iter() {
            writeln!(output_file, "Порт {}: {} - открыт", port, get_port_description(port))?;
        }
    }

    writeln!(output_file, "-----------------")?;

    if !api_requests.is_empty() {
        writeln!(output_file, "Запросы к API:")?;
        for request in api_requests.iter() {
            writeln!(output_file, "{}", request)?;
        }
    }

    println!("Рабочие ссылки:");
    for url in results.iter() {
        println!("{}", url);
    }

    // Вывод информации об открытых портах
    if !open_ports.is_empty() {
        println!("Открытые порты:");
        for &port in open_ports.iter() {
            println!("Порт {}: {} - открыт", port, get_port_description(port));
        }
    }

    // Вывод информации о запросах к API
    if !api_requests.is_empty() {
        println!("Запросы к API:");
        for request in api_requests.iter() {
            println!("{}", request);
        }
    }

    println!("Программа завершена за {:?}", duration);
    println!("Результаты сохранены в {}", output_filename);

    execute!(stdout(), LeaveAlternateScreen).unwrap();
    Ok(())
}
