import urllib.parse

import arrow
import requests


GITLAB_URL = 'https://gitlab.com'


def get_groups(token, insecure, base_url):
    api = GitLab(
        url=base_url,
        token=token)
    return sorted(api.groups())


def get_repos(path, token, insecure=False, base_url=None, archived=None,
              is_fork=None, all=False):
    api = GitLab(
        url=base_url,
        token=token)

    projects = []
    for project in api.projects(path, archived=archived, fork=is_fork):
        last_activity_at = project['last_activity_at']
        if last_activity_at is not None:
            last_activity_at = arrow.get(last_activity_at).datetime
        projects.append(
            (project['path_with_namespace'],
             project['ssh_url_to_repo'],
             last_activity_at,
             project['default_branch']))
    return sorted(projects)


class GitLab:

    def __init__(self, token=None, url=None):
        self.url = url if url is not None else 'https://gitlab.com'
        self.session = requests.Session()
        if token is not None:
            self.session.headers['Authorization'] = f'Bearer {token}'

    def groups(self):
        qs = urllib.parse.urlencode({
            'all_available': 'true',
            'order_by': 'id',
            'pagination': 'keyset',
            'per_page': '100'
        })
        url = f'{self.url}/api/v4/groups?{qs}'

        groups = set()
        while url:
            response = self.session.get(url)
            response.raise_for_status()

            for item in response.json():
                groups.add(item['full_path'])

            if next_link := response.links.get('next'):
                url = next_link['url']
            else:
                url = None

        return groups

    def projects(self, *args, **kwargs):
        try:
            return [self.project(*args, **kwargs)]
        except NotFound:
            try:
                return self.user_projects(*args, **kwargs)
            except NotFound:
                return self.group_projects(*args, **kwargs)

    def project(self, name, archived=None, fork=None):
        safename = urllib.parse.quote(name, safe='')
        url = f'{self.url}/api/v4/projects/{safename}'
        response = self.session.get(url)
        if response.status_code == 404:
            try:
                message = response.json()['message']
            except Exception:
                pass
            else:
                if message == '404 Project Not Found':
                    raise NotFound
        response.raise_for_status()
        project = response.json()
        if fork is not None:
            if fork != bool(project.get('forked_from_project')):
                raise NotFound
        if archived is not None and archived != project['archived']:
            raise NotFound
        path = project['path_with_namespace']
        if path.lower() == name.lower():
            return project

    def group_projects(self, *args, **kwargs):
        return self._projects('groups', *args, **kwargs)

    def user_projects(self, *args, **kwargs):
        return self._projects('users', *args, **kwargs)

    def _projects(self, name_type, name, archived=None, fork=None):
        name = urllib.parse.quote(name, safe='')

        query = {
            'include_subgroups': 'true',
            'order_by': 'id',
            'pagination': 'keyset',
            'per_page': '100'
        }
        if archived is not None:
            query['archived'] = str(archived).lower()

        qs = urllib.parse.urlencode(query)
        url = f'{self.url}/api/v4/{name_type}/{name}/projects?{qs}'

        projects = {}
        while url:
            response = self.session.get(url)
            if response.status_code == 404:
                try:
                    message = response.json()['message']
                except Exception:
                    pass
                else:
                    if message == '404 User Not Found':
                        raise NotFound('User Not Found')
                    if message == '404 Group Not Found':
                        raise NotFound('Group Not Found')
            response.raise_for_status()
            for project in response.json():
                if fork is not None:
                    if fork != bool(project.get('forked_from_project')):
                        continue
                projects[project['path_with_namespace']] = project

            if next_link := response.links.get('next'):
                url = next_link['url']
            else:
                url = None

        expected_prefix = f'{name.lower()}/'
        return [
            value
            for key, value in projects.items()
            if key.lower().startswith(expected_prefix)
        ]


class NotFound(Exception):
    """Raised when a user or group is not found"""
