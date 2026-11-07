// Copyright (c) 2026 jk9576 (Jeong Hoon Kim). All rights reserved.

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

// Extern Carbon analytical routines
extern int32_t ComputeErrorRateBps(int32_t errors, int32_t total);
extern int32_t ComputeAvgLatency(int32_t sum_latency, int32_t count);
extern int32_t LatencyToBucket(int32_t latency_ms);
extern int32_t ComputePercentileIndex(int32_t count, int32_t num, int32_t den);

static int compare_i32(const void *a, const void *b) {
    int32_t arg1 = *(const int32_t *)a;
    int32_t arg2 = *(const int32_t *)b;
    return (arg1 > arg2) - (arg1 < arg2);
}

int main(int argc, char **argv) {
    printf("==========================================================\n");
    printf(" Carbon Analytical Query Engine (Vectorized Aggregator)\n");
    printf("==========================================================\n");

    int32_t count = 0;
    int32_t *latencies = NULL;
    int32_t *status_codes = NULL;
    int32_t *levels = NULL;

    if (argc >= 2) {
        FILE *f = fopen(argv[1], "rb");
        if (!f) {
            fprintf(stderr, "Error: Unable to open data file '%s'\n", argv[1]);
            return 1;
        }

        char magic[4];
        if (fread(magic, 1, 4, f) != 4 || memcmp(magic, "CARB", 4) != 0) {
            fprintf(stderr, "Error: Invalid CARB file format magic\n");
            fclose(f);
            return 1;
        }

        uint32_t raw_count = 0;
        if (fread(&raw_count, 4, 1, f) != 1) {
            fclose(f);
            return 1;
        }
        count = (int32_t)raw_count;

        latencies = (int32_t *)malloc(count * sizeof(int32_t));
        status_codes = (int32_t *)malloc(count * sizeof(int32_t));
        levels = (int32_t *)malloc(count * sizeof(int32_t));

        fread(latencies, sizeof(int32_t), count, f);
        fread(status_codes, sizeof(int32_t), count, f);
        fread(levels, sizeof(int32_t), count, f);
        fclose(f);

        printf("Loaded %d columnar records from '%s'\n", count, argv[1]);
    } else {
        // Self-test mode with 10,000 synthetic records
        count = 10000;
        latencies = (int32_t *)malloc(count * sizeof(int32_t));
        status_codes = (int32_t *)malloc(count * sizeof(int32_t));
        levels = (int32_t *)malloc(count * sizeof(int32_t));

        for (int32_t i = 0; i < count; i++) {
            int32_t is_err = (i % 13 == 0);
            status_codes[i] = is_err ? 503 : 200;
            levels[i] = is_err ? 5 : 3;
            latencies[i] = is_err ? 320 : 12 + ((i * 17) % 180);
        }
        printf("Running Carbon analytics over %d synthetic records\n", count);
    }

    if (count == 0) {
        printf("No records to process.\n");
        free(latencies);
        free(status_codes);
        free(levels);
        return 0;
    }

    // Heavy aggregations powered by Carbon
    int32_t error_count = 0;
    int64_t sum_latency = 0;
    int32_t bucket_histogram[6] = {0};

    for (int32_t i = 0; i < count; i++) {
        if (status_codes[i] >= 500) {
            error_count++;
        }
        sum_latency += latencies[i];
        int32_t bucket = LatencyToBucket(latencies[i]);
        if (bucket >= 0 && bucket < 6) {
            bucket_histogram[bucket]++;
        }
    }

    // Sort latencies for percentile calculations
    qsort(latencies, count, sizeof(int32_t), compare_i32);

    int32_t p50_idx = ComputePercentileIndex(count, 50, 100);
    int32_t p95_idx = ComputePercentileIndex(count, 95, 100);
    int32_t p99_idx = ComputePercentileIndex(count, 99, 100);

    int32_t p50 = latencies[p50_idx];
    int32_t p95 = latencies[p95_idx];
    int32_t p99 = latencies[p99_idx];
    int32_t avg = ComputeAvgLatency((int32_t)(sum_latency / 1), count);
    int32_t err_bps = ComputeErrorRateBps(error_count, count);

    printf("\n--- Carbon Analytics Aggregation Results ---\n");
    printf("TOTAL_RECORDS:   %d\n", count);
    printf("MATCHED_RECORDS: %d\n", count);
    printf("ERROR_COUNT:     %d\n", error_count);
    printf("ERROR_RATE_BPS:  %d (%0.2f%%)\n", err_bps, (double)err_bps / 100.0);
    printf("LATENCY_AVG:     %d ms\n", avg);
    printf("LATENCY_P50:     %d ms\n", p50);
    printf("LATENCY_P95:     %d ms\n", p95);
    printf("LATENCY_P99:     %d ms\n", p99);

    printf("\n--- Latency Histogram Distribution ---\n");
    const char *labels[] = {
        "[  0ms ..  20ms)",
        "[ 20ms ..  50ms)",
        "[ 50ms .. 100ms)",
        "[100ms .. 250ms)",
        "[250ms .. 500ms)",
        "[500ms +        )"
    };
    for (int b = 0; b < 6; b++) {
        int pct = (bucket_histogram[b] * 100) / count;
        printf("  %-18s : %5d records (%3d%%)\n", labels[b], bucket_histogram[b], pct);
    }
    printf("--------------------------------------------\n");
    printf("Carbon Analytics Query Executed Successfully.\n");

    free(latencies);
    free(status_codes);
    free(levels);
    return 0;
}
