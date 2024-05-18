## Описание
dirb_rust - это утилита для проверки доступности URL-адресов и открытых портов, а также для поиска запросов к API на заданном сайте. Программа способна обрабатывать до полумиллиона строк всего за 3 минуты, что делает ее чрезвычайно эффективной для больших списков URL.

## Возможности
- Объединение всех текстовых файлов в директории wordlists в один файл, удаляя дубликаты.
- Асинхронная проверка доступности URL-адресов.
- Проверка открытых портов на заданном сайте.
- Поиск запросов к API (fetch, axios, XMLHttpRequest) на указанном сайте.
- Отображение результатов с использованием прогресс-бара и терминального интерфейса.
## Установка
1. Установите Rust, если он еще не установлен.
2. Клонируйте этот репозиторий:
```bash
[git clone https://github.com/your-username/url-scanner.git](https://github.com/Nopass0/dirb_rust.git)
cd url-scanner
```
## Использование
1. Поместите свои словари в формате .txt в директорию wordlists. Каждый словарь должен содержать строки с путями, которые необходимо проверить.
2. Запустите программу:
```bash
cargo run --release
```
3. Введите URL сайта, который хотите проверить (например, http://example.com).
Программа объединит все словари в один файл wordlist.txt, удаляя повторяющиеся строки, и начнет проверку доступности URL, открытых портов и запросов к API.

## Результаты
Результаты проверки будут сохранены в файл с именем output_{base_domain}_{current_date}.txt, где base_domain - это базовый домен проверяемого сайта, а current_date - текущая дата. В этом файле вы найдете:

Список доступных URL.
Открытые порты с их описанием.
Найденные запросы к API.
Пример вывода
```
Рабочие ссылки:
http://example.com/path1
http://example.com/path2
...

Открытые порты:
Порт 80: HTTP - открыт
Порт 443: HTTPS - открыт
...

Запросы к API:
fetch('http://example.com/api')
axios.get('http://example.com/api')
...
```
Системные требования
Rust 1.50 или выше.
Операционная система: Windows, Linux или macOS.

---------------------------------------------
## Description
dirb_rust is a utility for checking the availability of URLs and open ports, as well as searching for API requests on a given site. The program can process up to half a million lines in just 3 minutes, making it extremely efficient for large URL lists.

## Features
- Merges all text files in the wordlists directory into one file, removing duplicates.
- Asynchronous URL availability checking.
- Checking open ports on a given site.
- Searching for API requests (fetch, axios, XMLHttpRequest) on the specified site.
- Displaying results using a progress bar and terminal interface.
- Installation
- Install Rust if it is not already installed.
## Clone this repository:
```bash
git clone https://github.com/Nopass0/dirb_rust.git
cd dirb_rust
```
## Usage
1. Place your dictionaries in .txt format in the wordlists directory. Each dictionary should contain lines with paths to be checked.
2. Run the program:
```bash
cargo run --release
```
3. Enter the URL of the site you want to check (e.g., http://example.com).
The program will merge all dictionaries into one file wordlist.txt, removing duplicate lines, and start checking URL availability, open ports, and API requests.

## Results
The results of the check will be saved in a file named output_{base_domain}_{current_date}.txt, where base_domain is the base domain of the site being checked and current_date is the current date. In this file, you will find:

A list of available URLs.
Open ports with their descriptions.
Found API requests.
Example Output
```
Working links:
http://example.com/path1
http://example.com/path2
...

Open ports:
Port 80: HTTP - open
Port 443: HTTPS - open
...

API requests:
fetch('http://example.com/api')
axios.get('http://example.com/api')
...
```
System Requirements
Rust 1.50 or higher.
Operating System: Windows, Linux, or macOS.
