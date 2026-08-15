#!/bin/bash

# Запуск бэкенда
cargo run --package rumary-api --bin rumary-api &
BACKEND_PID=$!

# Запуск фронтенда
(cd frontend && npm run dev) &
FRONTEND_PID=$!

# При Ctrl+C или завершении скрипта — убить оба процесса
trap "echo 'Остановка...'; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null; exit" SIGINT SIGTERM

# Ждать завершения обоих
wait $BACKEND_PID $FRONTEND_PID