"""Bounded retry decisions for explicitly diagnosed anonymous GitHub limits."""
import math


def https_retry_plan(record, attempt, now):
    """Only a newly diagnosed anonymous GitHub API limit may delay another probe."""
    stop = lambda reason: dict(retry=False, reason=reason)
    if (type(record.get('http_diagnostics_schema')) is not int or
            record['http_diagnostics_schema'] != 1 or record.get('error') != 'rate_limited' or
            record.get('rate_limit_confirmed') is not True or
            record.get('confirmed_from_controller') is not True or record.get('timeout') is not False):
        return stop('not_confirmed_rate_limit')
    responses = record.get('http_responses')
    if not isinstance(responses, list) or not responses or not isinstance(responses[-1], dict):
        return stop('missing_http_response')
    requests = record.get('request_count')
    if (record.get('response_records_truncated') is not False or type(requests) is not int or
            requests < len(responses) or requests > 32):
        return stop('incomplete_http_diagnostics')
    response = responses[-1]
    status = response.get('status')
    if (type(status) is not int or status not in (403, 429) or
            response.get('host') != 'api.github.com' or response.get('route') != 'release_list' or
            record.get('authorization_present') is not False):
        return stop('not_anonymous_github_api_limit')
    headers = response.get('rate_headers')
    if not isinstance(headers, dict):
        return stop('missing_rate_headers')
    def integer_header(name):
        value = headers.get(name)
        if value is None:
            return None
        if (not isinstance(value, str) or not value.isascii() or not value.isdecimal()
                or len(value) > 20):
            raise ValueError(name)
        return int(value)
    try:
        remaining = integer_header('remaining')
        retry_after = integer_header('retry_after')
    except ValueError:
        return stop('invalid_rate_header')
    delay, reason, clock = None, None, {}
    if remaining == 0:
        try:
            reset = integer_header('reset')
        except ValueError:
            return stop('invalid_primary_reset')
        if reset is None:
            return stop('missing_primary_reset')
        server_time = response.get('server_date_epoch')
        server_clock = type(server_time) is int and server_time > 0
        current = server_time if server_clock else now
        if type(current) not in (int, float) or not math.isfinite(current) or current <= 0:
            return stop('invalid_reference_clock')
        if reset <= current:
            return stop('primary_reset_not_future')
        delay, reason = math.ceil(reset - current) + 1, 'primary_reset'
        clock = dict(reference_epoch=current, reference_clock='http_date' if server_clock else 'runner_wall_clock')
    if retry_after is not None:
        delay = max(delay or 1, retry_after, 1)
        reason = 'retry_after_and_primary_reset' if reason else 'retry_after'
    if delay is None:
        # Controller confirms explicit rate-limit responses; plain 403 never
        # reaches this branch. GitHub documents at least one minute without a
        # server deadline, followed by exponential backoff.
        delay, reason = 60 * 2 ** (attempt - 1), 'confirmed_limit_exponential_backoff'
    return dict(retry=True, reason=reason, delay_seconds=delay, **clock)
